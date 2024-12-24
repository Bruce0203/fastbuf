use core::{
    fmt::Debug,
    mem::MaybeUninit,
    ops::{Index, IndexMut, Range},
    ptr,
};
use std::{
    alloc::{Allocator, Global},
    ptr::slice_from_raw_parts_mut,
};

use crate::{Buf, ReadBuf, ReadToBuf, WriteBuf, WriteBufferError};

enum RawBuffer<T, const N: usize, A: Allocator = Global> {
    Slice([T; N]),
    Boxed(Box<[T; N], A>),
}

impl<T, const N: usize> RawBuffer<T, N> {
    #[inline]
    pub fn new_boxed() -> Self {
        Self::Boxed(unsafe { Box::new_uninit().assume_init() })
    }
}

impl<A: Allocator, T, const N: usize> RawBuffer<T, N, A> {
    #[inline]
    pub fn new() -> Self {
        Self::Slice(unsafe { MaybeUninit::uninit().assume_init() })
    }

    #[inline]
    pub fn new_boxed_in(alloc: A) -> Self {
        Self::Boxed(unsafe { Box::new_uninit_in(alloc).assume_init() })
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        match self {
            RawBuffer::Slice(slice) => slice.as_ptr(),
            RawBuffer::Boxed(boxed) => boxed.as_ptr(),
        }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        match self {
            RawBuffer::Slice(slice) => slice.as_mut_ptr(),
            RawBuffer::Boxed(boxed) => boxed.as_mut_ptr(),
        }
    }

    #[inline]
    pub fn to_slice(&self) -> &[T; N] {
        match self {
            RawBuffer::Slice(slice) => slice,
            RawBuffer::Boxed(boxed) => &**boxed,
        }
    }

    #[inline]
    pub fn to_slice_mut(&mut self) -> &mut [T; N] {
        match self {
            RawBuffer::Slice(slice) => slice,
            RawBuffer::Boxed(boxed) => &mut **boxed,
        }
    }
}

impl<T, const N: usize, A: Allocator> Index<Range<usize>> for RawBuffer<T, N, A> {
    type Output = [T];

    #[inline]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        self.to_slice().index(index)
    }
}

impl<T, const N: usize, A: Allocator> IndexMut<Range<usize>> for RawBuffer<T, N, A> {
    #[inline]
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        self.to_slice_mut().index_mut(index)
    }
}

pub struct Buffer<const N: usize, A: Allocator = Global> {
    chunk: RawBuffer<u8, N, A>,
    filled_pos: LenUint,
    pos: LenUint,
}

type LenUint = u32;

impl<const N: usize> Buffer<N> {
    #[inline]
    pub fn new() -> Self {
        Self {
            chunk: RawBuffer::new(),
            filled_pos: 0,
            pos: 0,
        }
    }

    #[inline]
    pub fn new_boxed() -> Self {
        Self {
            chunk: RawBuffer::new_boxed(),
            filled_pos: 0,
            pos: 0,
        }
    }

    #[inline]
    pub fn to_slice(&self) -> &[u8; N] {
        self.chunk.to_slice()
    }

    #[inline]
    pub fn to_slice_mut(&mut self) -> &mut [u8; N] {
        self.chunk.to_slice_mut()
    }
}

impl<const N: usize> Buf for Buffer<N> {
    #[inline]
    fn clear(&mut self) {
        self.filled_pos = 0;
        self.pos = 0;
    }

    #[inline]
    fn as_ptr(&self) -> *const u8 {
        self.chunk.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.chunk.as_mut_ptr()
    }

    #[inline]
    fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> WriteBuf for Buffer<N> {
    #[inline]
    fn try_write(&mut self, data: &[u8]) -> Result<(), WriteBufferError> {
        let filled_pos = self.filled_pos as usize;
        let len = data.len();
        let new_filled_pos_len = filled_pos + len;
        if new_filled_pos_len > N {
            return Err(WriteBufferError::BufferFull);
        }
        self.filled_pos = new_filled_pos_len as LenUint;
        let dst = unsafe { &mut *slice_from_raw_parts_mut(self.chunk.as_mut_ptr(), len) };
        dst.copy_from_slice(data);
        Ok(())
    }

    #[inline]
    fn write(&mut self, data: &[u8]) {
        let filled_pos = self.filled_pos as usize;
        let new_filled_pos_len = filled_pos + data.len();
        self.chunk[filled_pos..new_filled_pos_len].copy_from_slice(data);
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

impl<const N: usize> ReadBuf for Buffer<N> {
    #[inline]
    fn read(&mut self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let slice_len = core::cmp::min(len, self.filled_pos as usize - pos);
        let new_pos = pos + slice_len;
        self.pos = new_pos as LenUint;
        unsafe { &*ptr::slice_from_raw_parts(self.chunk.as_ptr().wrapping_add(pos), slice_len) }
    }

    #[inline]
    unsafe fn get_continuous(&self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let filled_pos = self.filled_pos as usize;
        let slice_len = core::cmp::min(len, filled_pos - pos);
        unsafe { &*ptr::slice_from_raw_parts(self.chunk.as_ptr().wrapping_add(pos), slice_len) }
    }

    #[inline]
    fn remaining(&self) -> usize {
        (self.filled_pos - self.pos) as usize
    }

    #[inline]
    fn advance(&mut self, len: usize) {
        let pos = self.pos as usize;
        self.pos = core::cmp::min(self.filled_pos, (pos + len) as LenUint);
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos as usize
    }

    #[inline]
    unsafe fn set_pos(&mut self, value: usize) {
        self.pos = value as u32;
    }
}

impl<T: std::io::Read> ReadToBuf for T {
    #[inline]
    fn read_to_buf(&mut self, buf: &mut impl Buf) -> Result<(), ()> {
        let filled_pos = buf.filled_pos() as usize;
        let slice = unsafe {
            &mut *slice_from_raw_parts_mut(
                buf.as_mut_ptr().wrapping_add(filled_pos),
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

impl<const N: usize> std::io::Write for Buffer<N> {
    #[inline]
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

impl<const N: usize> Debug for Buffer<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.chunk[self.pos()..self.filled_pos()].fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use test::{black_box, Bencher};

    use super::*;

    #[test]
    fn test_debug() {
        let f = |mut buffer: Buffer<16>| {
            let data = b"test";

            buffer.write(data);
            let debug_str = format!("{:?}", buffer);
            assert_eq!(debug_str, "[116, 101, 115, 116]");
        };
        f(Buffer::new());
        f(Buffer::new_boxed());
    }

    #[test]
    fn test_write_and_read() {
        let f = |mut buffer: Buffer<16>| {
            let data = b"hello";

            buffer.write(data);
            assert_eq!(buffer.remaining_space(), 11);

            let read_data = buffer.read(5);
            assert_eq!(read_data, data);
        };
        f(Buffer::new());
        f(Buffer::new_boxed());
    }

    #[test]
    fn test_try_write_success() {
        let f = |mut buffer: Buffer<16>| {
            let data = b"hello";

            assert!(buffer.try_write(data).is_ok());
            assert_eq!(buffer.remaining_space(), 11);
        };
        f(Buffer::new());
        f(Buffer::new_boxed());
    }

    #[test]
    fn test_try_write_fail() {
        let f = |mut buffer: Buffer<8>| {
            let data = b"too long data";

            assert!(buffer.try_write(data).is_err());
            assert_eq!(buffer.remaining_space(), 8);
            buffer.try_write(&[]).unwrap();
        };
        f(Buffer::new());
        f(Buffer::new_boxed());
    }

    #[test]
    fn test_clear() {
        let f = |mut buffer: Buffer<16>| {
            let data = b"hello";

            buffer.write(data);
            buffer.clear();
            assert_eq!(buffer.remaining_space(), 16);
            assert_eq!(buffer.remaining(), 0);
        };
        f(Buffer::new());
        f(Buffer::new_boxed());
    }

    #[test]
    fn test_advance() {
        let f = |mut buffer: Buffer<16>| {
            let data = b"hello world";

            buffer.write(data);
            buffer.advance(6);
            assert_eq!(buffer.remaining(), 5);

            let remaining_data = buffer.read(5);
            assert_eq!(remaining_data, b"world");
        };
        f(Buffer::new());
        f(Buffer::new_boxed());
    }

    #[test]
    fn test_get_continuous() {
        let f = |mut buffer: Buffer<16>| {
            let data = b"hello world";

            buffer.write(data);
            let continuous_data = unsafe { buffer.get_continuous(5) };
            assert_eq!(continuous_data, b"hello");
        };
        f(Buffer::new());
        f(Buffer::new_boxed());
    }

    const N: usize = 1000;
    const SAMPLE_SIZE: usize = 1000;

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
