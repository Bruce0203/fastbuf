use core::{
    fmt::Debug,
    mem::{transmute_copy, MaybeUninit},
    ptr,
};
use std::ptr::slice_from_raw_parts_mut;

use crate::{Buf, ReadBuf, ReadToBuf, WriteBuf, WriteBufferError};

pub struct Buffer<const N: usize> {
    chunk: [u8; N],
    filled_pos: LenUint,
    pos: LenUint,
}

type LenUint = u32;

impl<const N: usize> Buffer<N> {
    pub fn new() -> Self {
        let chunk = unsafe { transmute_copy(&MaybeUninit::<[u8; N]>::uninit()) };
        Self {
            chunk,
            filled_pos: 0,
            pos: 0,
        }
    }

    pub fn new_boxed() -> Box<Self> {
        let box_uninit = Box::<Self>::new_uninit();
        unsafe {
            let mut box_uninit = box_uninit.assume_init();
            box_uninit.set_pos(0);
            box_uninit.set_filled_pos(0);
            box_uninit
        }
    }

    pub fn to_slice(&self) -> &[u8; N] {
        &self.chunk
    }

    pub fn to_slice_mut(&mut self) -> &mut [u8; N] {
        &mut self.chunk
    }
}

impl<const N: usize> Buf for Buffer<N> {
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

impl<const N: usize> WriteBuf for Buffer<N> {
    fn try_write(&mut self, data: &[u8]) -> Result<(), WriteBufferError> {
        let filled_pos = self.filled_pos as usize;
        let new_filled_pos_len = filled_pos + data.len();
        if new_filled_pos_len > N {
            return Err(WriteBufferError::BufferFull);
        }
        let dst = unsafe { self.chunk.get_unchecked_mut(filled_pos..new_filled_pos_len) };
        dst.copy_from_slice(data);
        self.filled_pos = new_filled_pos_len as LenUint;
        Ok(())
    }

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
    fn read(&mut self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let slice_len = core::cmp::min(len, self.filled_pos as usize - pos);
        let new_pos = pos + slice_len;
        self.pos = new_pos as LenUint;
        unsafe { &*ptr::slice_from_raw_parts(self.chunk.as_ptr().wrapping_add(pos), slice_len) }
    }

    unsafe fn get_continuous(&self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let filled_pos = self.filled_pos as usize;
        let slice_len = core::cmp::min(len, filled_pos - pos);
        unsafe { &*ptr::slice_from_raw_parts(self.chunk.as_ptr().wrapping_add(pos), slice_len) }
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
mod test {
    use ::test::{black_box, Bencher};

    use super::*;

    #[test]
    fn test_debug() {
        let mut buffer: Buffer<16> = Buffer::new();
        let data = b"test";

        buffer.write(data);
        let debug_str = format!("{:?}", buffer);
        assert_eq!(debug_str, "[116, 101, 115, 116]");
    }

    #[test]
    fn test_write_and_read() {
        let mut buffer: Buffer<16> = Buffer::new();
        let data = b"hello";

        buffer.write(data);
        assert_eq!(buffer.remaining_space(), 11);

        let read_data = buffer.read(5);
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_try_write_success() {
        let mut buffer: Buffer<16> = Buffer::new();
        let data = b"hello";

        assert!(buffer.try_write(data).is_ok());
        assert_eq!(buffer.remaining_space(), 11);
    }

    #[test]
    fn test_try_write_fail() {
        let mut buffer: Buffer<8> = Buffer::new();
        let data = b"too long data";

        assert!(buffer.try_write(data).is_err());
        assert_eq!(buffer.remaining_space(), 8);
    }

    #[test]
    fn test_clear() {
        let mut buffer: Buffer<16> = Buffer::new();
        let data = b"hello";

        buffer.write(data);
        buffer.clear();
        assert_eq!(buffer.remaining_space(), 16);
        assert_eq!(buffer.remaining(), 0);
    }

    #[test]
    fn test_advance() {
        let mut buffer: Buffer<16> = Buffer::new();
        let data = b"hello world";

        buffer.write(data);
        buffer.advance(6);
        assert_eq!(buffer.remaining(), 5);

        let remaining_data = buffer.read(5);
        assert_eq!(remaining_data, b"world");
    }

    #[test]
    fn test_get_continuous() {
        let mut buffer: Buffer<16> = Buffer::new();
        let data = b"hello world";

        buffer.write(data);
        let continuous_data = unsafe { buffer.get_continuous(5) };
        assert_eq!(continuous_data, b"hello");
    }

    #[test]
    fn test_buffer_try_write_until_full() {
        let mut buffer: Buffer<16> = Buffer::new();
        let src: &[u8] = &vec![0; buffer.capacity()];
        buffer.try_write(src).unwrap();
        buffer.try_write(&[]).unwrap();
        buffer.try_write(&[0]).unwrap_err();
    }

    const N: usize = 1000;

    #[bench]
    fn bench_buffer_try_write(b: &mut Bencher) {
        let ref mut buffer: Buffer<N> = Buffer::new();
        let src: &[u8] = &vec![0; N];
        b.iter(|| {
            unsafe { buffer.set_filled_pos(0) };
            let _ = black_box(&buffer.try_write(black_box(&src)));
        });
        black_box(&buffer);
    }

    #[bench]
    fn bench_buffer_write(b: &mut Bencher) {
        let ref mut buffer: Buffer<N> = Buffer::new();
        let src: &[u8] = &vec![0; N];
        b.iter(|| {
            unsafe { buffer.set_filled_pos(0) };
            let _ = black_box(&buffer.write(black_box(&src)));
        });
        black_box(&buffer);
    }
}
