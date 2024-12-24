use core::{
    fmt::Debug,
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    ops::{Deref, DerefMut, Index, IndexMut, Range},
    ptr,
};
use std::{
    alloc::{Allocator, Global},
    ptr::slice_from_raw_parts_mut,
};

use crate::{Buf, ReadBuf, ReadToBuf, WriteBuf, WriteBufferError};

pub type BoxedBuffer<const N: usize, A: Allocator = Global> = Buffer<N, A, Box<[u8; N]>>;

pub struct Buffer<const N: usize, A: Allocator = Global, C: Chunk<u8, N, A> = [u8; N]> {
    chunk: C,
    filled_pos: LenUint,
    pos: LenUint,
    _spooky: PhantomData<A>,
}

pub trait Chunk<T, const N: usize, A: Allocator> {
    fn new_uninit_in(alloc: A) -> Self;
    fn as_slice(&self) -> &[T; N];
    fn as_mut_slice(&mut self) -> &mut [T; N];
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for [T; N] {
    fn as_slice(&self) -> &[T; N] {
        self
    }

    fn as_mut_slice(&mut self) -> &mut [T; N] {
        self
    }

    fn new_uninit_in(_alloc: A) -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    fn as_ptr(&self) -> *const T {
        <[T]>::as_ptr(self)
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        <[T]>::as_mut_ptr(self)
    }
}

impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for Box<[T; N], A> {
    fn as_slice(&self) -> &[T; N] {
        self
    }

    fn as_mut_slice(&mut self) -> &mut [T; N] {
        self
    }

    fn new_uninit_in(alloc: A) -> Self {
        unsafe { Box::new_uninit_in(alloc).assume_init() }
    }

    fn as_ptr(&self) -> *const T {
        <[T]>::as_ptr(self.deref())
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        <[T]>::as_mut_ptr(self.deref_mut())
    }
}

type LenUint = u32;

impl<A: Allocator, const N: usize, C: Chunk<u8, N, A>> Buffer<N, A, C> {
    pub fn new_in(alloc: A) -> Self {
        Self {
            chunk: C::new_uninit_in(alloc),
            filled_pos: 0,
            pos: 0,
            _spooky: PhantomData,
        }
    }
}

impl<const N: usize, C: Chunk<u8, N, Global>> Buffer<N, Global, C> {
    pub fn new() -> Self {
        Self {
            chunk: C::new_uninit_in(Global),
            filled_pos: 0,
            pos: 0,
            _spooky: PhantomData,
        }
    }
}

impl<const N: usize, A: Allocator, C: Chunk<u8, N, A>> Buf for Buffer<N, A, C> {
    fn clear(&mut self) {
        self.filled_pos = 0;
        self.pos = 0;
    }

    fn as_ptr(&self) -> *const u8 {
        self.chunk.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.chunk.as_mut_ptr()
    }

    fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize, A: Allocator, C: Chunk<u8, N, A>> WriteBuf for Buffer<N, A, C> {
    fn try_write(&mut self, data: &[u8]) -> Result<(), WriteBufferError> {
        let filled_pos = self.filled_pos as usize;
        let new_filled_pos = filled_pos + data.len();
        if new_filled_pos <= N {
            unsafe {
                self.chunk
                    .as_mut_slice()
                    .get_unchecked_mut(filled_pos..new_filled_pos)
                    .copy_from_slice(data);
            }
            self.filled_pos = new_filled_pos as LenUint;
            Ok(())
        } else {
            Err(WriteBufferError::BufferFull)
        }
    }

    fn write(&mut self, data: &[u8]) {
        let filled_pos = self.filled_pos as usize;
        let new_filled_pos_len = filled_pos + data.len();
        self.chunk.as_mut_slice()[filled_pos..new_filled_pos_len].copy_from_slice(data);
        self.filled_pos = new_filled_pos_len as LenUint;
    }

    fn remaining_space(&self) -> usize {
        N - self.filled_pos as usize
    }

    fn filled_pos(&self) -> usize {
        self.filled_pos as usize
    }

    unsafe fn set_filled_pos(&mut self, value: usize) {
        self.filled_pos = value as u32;
    }
}

impl<const N: usize, A: Allocator, C: Chunk<u8, N, A>> ReadBuf for Buffer<N, A, C> {
    fn read(&mut self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let slice_len = core::cmp::min(len, self.filled_pos as usize - pos);
        let new_pos = pos + slice_len;
        self.pos = new_pos as LenUint;
        unsafe { &*ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), slice_len) }
    }

    unsafe fn get_continuous(&self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let filled_pos = self.filled_pos as usize;
        let slice_len = core::cmp::min(len, filled_pos - pos);
        unsafe { &*ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), slice_len) }
    }

    fn remaining(&self) -> usize {
        (self.filled_pos - self.pos) as usize
    }

    fn advance(&mut self, len: usize) {
        let pos = self.pos as usize;
        self.pos = core::cmp::min(self.filled_pos, (pos + len) as LenUint);
    }

    fn pos(&self) -> usize {
        self.pos as usize
    }

    unsafe fn set_pos(&mut self, value: usize) {
        self.pos = value as u32;
    }
}

impl<T: std::io::Read> ReadToBuf for T {
    fn read_to_buf(&mut self, buf: &mut impl Buf) -> Result<(), ()> {
        let filled_pos = buf.filled_pos() as usize;
        let slice = unsafe {
            &mut *slice_from_raw_parts_mut(
                buf.as_mut_ptr().offset(filled_pos as isize),
                buf.capacity() - filled_pos,
            )
        };
        let read_length = self.read(slice).map_err(|_| ())?;
        if read_length == 0 {
            Err(())?
        }
        unsafe { buf.set_filled_pos(filled_pos + read_length) };
        Ok(())
    }
}

impl<const N: usize, A: Allocator, C: Chunk<u8, N, A>> std::io::Write for Buffer<N, A, C> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let backup_filled_pos = self.filled_pos();
        self.try_write(buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "write buffer failed"))?;
        Ok(self.filled_pos() - backup_filled_pos)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<const N: usize, A: Allocator, C: Chunk<u8, N, A>> Debug for Buffer<N, A, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.chunk.as_slice()[self.pos()..self.filled_pos()].fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use test::{black_box, Bencher};

    use super::*;

    #[test]
    fn test_debug() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
                let data = b"test";

                buffer.write(data);
                let debug_str = format!("{:?}", buffer);
                assert_eq!(debug_str, "[116, 101, 115, 116]");
            }
        }
        test!(Buffer::<16>::new());
        test!(BoxedBuffer::<16, Global>::new());
    }

    #[test]
    fn test_write_and_read() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello";

            buffer.write(data);
            assert_eq!(buffer.remaining_space(), 11);

            let read_data = buffer.read(5);
            assert_eq!(read_data, data);
            }
        }
        test!(Buffer::<16>::new());
        test!(BoxedBuffer::<16, Global>::new());
    }

    #[test]
    fn test_try_write_success() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello";

            assert!(buffer.try_write(data).is_ok());
            assert_eq!(buffer.remaining_space(), 11);
        }
        }
        
        test!(Buffer::<16>::new());
        test!(BoxedBuffer::<16, Global>::new());
    }

    #[test]
    fn test_try_write_fail() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"too long data";

            assert!(buffer.try_write(data).is_err());
            assert_eq!(buffer.remaining_space(), 8);
            buffer.try_write(&[]).unwrap();
        }}
        
        test!(Buffer::<8>::new());
        test!(BoxedBuffer::<8, Global>::new());
    }

    #[test]
    fn test_clear() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello";

            buffer.write(data);
            buffer.clear();
            assert_eq!(buffer.remaining_space(), 16);
            assert_eq!(buffer.remaining(), 0);
        }}
        
        test!(Buffer::<16>::new());
        test!(BoxedBuffer::<16, Global>::new());
    }

    #[test]
    fn test_advance() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello world";

            buffer.write(data);
            buffer.advance(6);
            assert_eq!(buffer.remaining(), 5);

            let remaining_data = buffer.read(5);
            assert_eq!(remaining_data, b"world");
        }}
        
        test!(Buffer::<16>::new());
        test!(BoxedBuffer::<16, Global>::new());
    }

    #[test]
    fn test_get_continuous() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello world";

            buffer.write(data);
            let continuous_data = unsafe { buffer.get_continuous(5) };
            assert_eq!(continuous_data, b"hello");
        }}
        
        test!(Buffer::<16>::new());
        test!(BoxedBuffer::<16, Global>::new());
    }

    const N: usize = 1000;

    #[bench]
    fn bench_buffer_try_write(b: &mut Bencher) {
        let ref mut buffer: Buffer<N> = Buffer::new();
        let src: &[u8] = &vec![0; N];
        black_box(&src);
        b.iter(|| {
            unsafe { buffer.set_filled_pos(0) };
            let _ = black_box(&buffer.try_write(&src));
        });
        black_box(&buffer);
    }

    #[bench]
    fn bench_buffer_write(b: &mut Bencher) {
        let ref mut buffer: Buffer<N> = Buffer::new();
        let src: &[u8] = &vec![0; N];
        black_box(&src);
        b.iter(|| {
            unsafe { buffer.set_filled_pos(0) };
            let _ = black_box(&buffer.write(&src));
        });
        black_box(&buffer);
    }
}
