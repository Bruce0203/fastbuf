use std::mem::{transmute_copy, MaybeUninit};

type LenUint = u32;

pub trait WriteBuf {
    fn write(&mut self, data: &[u8]) -> Result<(), ()>;
    fn fill<F>(&mut self, len: usize, f: F)
    where
        F: FnOnce(&mut [u8]) -> usize;
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

    fn fill<F>(&mut self, len: usize, f: F)
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        let filled_pos = self.filled_pos as usize;
        let slice_len = std::cmp::min(len, N - filled_pos);
        unsafe {
            let slice = &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.as_mut_ptr().offset(filled_pos as isize),
                slice_len,
            );
            self.filled_pos = (filled_pos + f(slice)) as LenUint;
        }
    }
}

impl<const N: usize> ReadBuf for Buffer<N> {
    fn read(&mut self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let slice_len = std::cmp::min(len, self.filled_pos as usize - pos);
        let new_pos = pos + slice_len;
        self.pos = new_pos as LenUint;
        unsafe {
            &*core::ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), slice_len)
        }
    }

    fn advance(&mut self, len: usize) {
        let pos = self.pos as usize;
        self.pos = std::cmp::min(self.filled_pos, (pos + len) as LenUint);
    }

    fn get_continuous(&self, len: usize) -> &[u8] {
        let pos = self.pos as usize;
        let filled_pos = self.filled_pos as usize;
        let slice_len = std::cmp::min(len, filled_pos - pos);
        unsafe {
            &*core::ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), slice_len)
        }
    }

    fn remaining(&self) -> usize {
        (self.filled_pos - self.pos) as usize
    }
}
