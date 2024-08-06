use std::mem::{transmute_copy, MaybeUninit};

pub trait WriteBuf {
    fn write(&mut self, data: &[u8]) -> Result<(), ()>;
}

pub trait ReadBuf {
    fn read(&mut self, len: usize) -> &[u8];
    fn advance(&mut self, len: usize);
    unsafe fn unfilled(&mut self, len: usize) -> &[u8];
}

pub struct Buffer<const N: usize> {
    pub chunk: [u8; N],
    pub filled_pos: usize,
    pub pos: usize,
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
        let new_filled_pos_len = self.filled_pos + data.len();
        if new_filled_pos_len < N {
            self.chunk[self.filled_pos..new_filled_pos_len].copy_from_slice(data);
            self.filled_pos = data.len();
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<const N: usize> ReadBuf for Buffer<N> {
    fn read(&mut self, len: usize) -> &[u8] {
        let pos = self.pos;
        let new_pos = pos + len;
        self.pos = new_pos;
        unsafe {
            &*core::ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), new_pos)
        }
    }

    fn advance(&mut self, len: usize) {
        let pos = self.pos;
        self.pos = pos + len;
    }

    unsafe fn unfilled(&mut self, len: usize) -> &[u8] {
        let pos = self.pos;
        let new_pos = pos + len;
        unsafe {
            &*core::ptr::slice_from_raw_parts(self.chunk.as_ptr().offset(pos as isize), new_pos)
        }
    }
}
