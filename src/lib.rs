extern crate core as std;

use core::{
    fmt::Debug,
    mem::{transmute_copy, MaybeUninit},
};

type LenUint = u32;

pub trait WriteBuf {
    fn write(&mut self, data: &[u8]) -> Result<(), ()>;
    fn write_many(&mut self, data: &[&[u8]]) -> Result<(), ()>;
}

pub trait ReadBuf {
    fn read(&mut self, len: usize) -> &[u8];
    fn advance(&mut self, len: usize);
    fn get_continuous(&self, len: usize) -> &[u8];
    fn remaining(&self) -> usize;
}

pub struct Buffer<const N: usize> {
    pub chunk: [u8; N],
    pub filled_pos: LenUint,
    pub pos: LenUint,
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

impl<const N: usize> WriteBuf for Buffer<N> {
    fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        let filled_pos = self.filled_pos as usize;
        let new_filled_pos_len = filled_pos + data.len();
        if new_filled_pos_len < N {
            self.chunk[filled_pos..new_filled_pos_len].copy_from_slice(data);
            self.filled_pos = new_filled_pos_len as LenUint;
            Ok(())
        } else {
            Err(())
        }
    }

    fn write_many(&mut self, data: &[&[u8]]) -> Result<(), ()> {
        let mut filled_pos = self.filled_pos as usize;
        let mut new_filled_pos_len = filled_pos;
        {
            let filled_pos = filled_pos;
            let mut new_filled_pos_len = filled_pos;
            for data in data.iter() {
                new_filled_pos_len += data.len();
            }
            if new_filled_pos_len >= N {
                return Err(());
            }
        }
        for data in data.iter() {
            filled_pos = new_filled_pos_len;
            new_filled_pos_len += data.len();
            self.chunk[filled_pos..new_filled_pos_len].copy_from_slice(data);
        }
        self.filled_pos = new_filled_pos_len as LenUint;
        Ok(())
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

#[cfg(test)]
mod test {
    //TODO
}
