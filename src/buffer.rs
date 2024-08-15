use core::{
    fmt::Debug,
    mem::{transmute_copy, MaybeUninit},
};
use std::io;

use crate::{Buf, ReadBuf, WriteBuf};

type LenUint = u32;

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
    pub fn new() -> Self {
        Self {
            chunk: unsafe { transmute_copy(&MaybeUninit::<[u8; N]>::uninit()) },
            filled_pos: 0,
            pos: 0,
        }
    }
}
impl<const N: usize> Buf for Buffer<N> {
    fn clear(&mut self) {
        self.filled_pos = 0;
        self.pos = 0;
    }

    fn pos(&self) -> usize {
        self.pos as usize
    }

    unsafe fn set_pos(&mut self, value: usize) {
        self.pos = value as u32;
    }

    fn filled_pos(&self) -> usize {
        self.filled_pos as usize
    }

    unsafe fn set_filled_pos(&mut self, value: usize) {
        self.filled_pos = value as u32;
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
        if read_length == 0 {
            Err(())?
        }
        buf.filled_pos = (filled_pos + read_length) as u32;
        Ok(())
    }
}

impl<const N: usize> std::io::Write for Buffer<N> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let backup_filled_pos = self.filled_pos();
        self.try_write(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "write buffer failed"))?;
        Ok(self.filled_pos() - backup_filled_pos)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
