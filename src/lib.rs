use core::{
    fmt::Debug,
    mem::{transmute_copy, MaybeUninit},
};

type LenUint = u32;

pub trait WriteBuf {
    fn write(&mut self, data: &[u8]);
    fn try_write(&mut self, data: &[u8]) -> Result<(), ()>;
    fn remaining_space(&self) -> usize;
}

pub trait ReadBuf {
    fn read(&mut self, len: usize) -> &[u8];
    fn advance(&mut self, len: usize);
    fn get_continuous(&self, len: usize) -> &[u8];
    fn remaining(&self) -> usize;
}

pub struct Buffer<const N: usize> {
    chunk: [u8; N],
    filled_pos: LenUint,
    pos: LenUint,
}

impl<const N: usize> Debug for Buffer<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.chunk[self.pos as usize..self.filled_pos as usize].fmt(f)
    }
}

impl<const N: usize> Buffer<N> {
    pub fn clear(&mut self) {
        self.filled_pos = 0;
        self.pos = 0;
    }

    pub fn new() -> Self {
        Self {
            chunk: unsafe { transmute_copy(&MaybeUninit::<[u8; N]>::uninit()) },
            filled_pos: 0,
            pos: 0,
        }
    }
}

impl<const N: usize> WriteBuf for Buffer<N> {
    fn try_write(&mut self, data: &[u8]) -> Result<(), ()> {
        let filled_pos = self.filled_pos as usize;
        let new_filled_pos_len = filled_pos + data.len();
        if new_filled_pos_len < N {
            let dst = unsafe { self.chunk.get_unchecked_mut(filled_pos..new_filled_pos_len) };
            dst.copy_from_slice(data);
            self.filled_pos = new_filled_pos_len as LenUint;
            Ok(())
        } else {
            Err(())
        }
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
}

impl<const N: usize> ReadBuf for Buffer<N> {
    fn read(&mut self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let slice_len = core::cmp::min(len, self.filled_pos as usize - pos);
        let new_pos = pos + slice_len;
        self.pos = new_pos as LenUint;
        unsafe {
            &*core::ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), slice_len)
        }
    }

    fn advance(&mut self, len: usize) {
        let pos = self.pos as usize;
        self.pos = core::cmp::min(self.filled_pos, (pos + len) as LenUint);
    }

    fn get_continuous(&self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let filled_pos = self.filled_pos as usize;
        let slice_len = core::cmp::min(len, filled_pos - pos);
        unsafe {
            &*core::ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), slice_len)
        }
    }

    fn remaining(&self) -> usize {
        (self.filled_pos - self.pos) as usize
    }
}

pub trait ReadToBuf {
    fn read_to_buf<const N: usize>(&mut self, buf: &mut Buffer<N>) -> Result<(), ()>;
}

impl<T: std::io::Read> ReadToBuf for T {
    fn read_to_buf<const N: usize>(&mut self, buf: &mut Buffer<N>) -> Result<(), ()> {
        let filled_pos = buf.filled_pos as usize;
        let slice = unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                buf.chunk.as_mut_ptr().offset(filled_pos as isize),
                N - filled_pos,
            )
        };
        let read_length = self.read(slice).map_err(|_| ())?;
        buf.filled_pos = (filled_pos + read_length) as u32;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        let continuous_data = buffer.get_continuous(5);
        assert_eq!(continuous_data, b"hello");
    }

    #[test]
    fn test_debug() {
        let mut buffer: Buffer<16> = Buffer::new();
        let data = b"test";

        buffer.write(data);
        let debug_str = format!("{:?}", buffer);
        assert_eq!(debug_str, "[116, 101, 115, 116]");
    }
}
