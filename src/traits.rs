use core::ops::{Deref, DerefMut};

use crate::{declare_impl, declare_trait};

declare_trait! {
    pub trait Buf<()>: (ReadBuf, WriteBuf) {
        fn clear(&mut self);
        fn as_ptr(&self) -> *const u8;
        fn as_mut_ptr(&mut self) -> *mut u8;
        fn capacity(&self) -> usize;
    }
}

declare_trait! {
    pub trait WriteBuf<()>: () {
        fn write(&mut self, data: &[u8]);
        fn try_write(&mut self, data: &[u8]) -> Result<(), WriteBufferError>;
        fn remaining_space(&self) -> usize;
        fn filled_pos(&self) -> usize;
        unsafe fn set_filled_pos(&mut self, filled_pos: usize);
    }
}

declare_trait! {
    pub trait ReadBuf<()>: () {
        fn read(&mut self, len: usize) -> &[u8];
        unsafe fn get_continuous(&self, len: usize) -> &[u8];
        fn remaining(&self) -> usize;
        fn advance(&mut self, len: usize);
        fn pos(&self) -> usize;
        unsafe fn set_pos(&mut self, pos: usize);
    }
}

pub trait ReadToBuf {
    #[cfg(feature = "const-trait")]
    fn read_to_buf(&mut self, buf: &mut impl const Buf) -> Result<(), ()>;
    #[cfg(not(feature = "const-trait"))]
    fn read_to_buf(&mut self, buf: &mut impl Buf) -> Result<(), ()>;
}

#[derive(Debug)]
pub enum WriteBufferError {
    BufferFull,
}

declare_impl! {
    (impl<T: Buf>  Buf for &mut T),
    (impl<T: const Buf> const Buf for &mut T) {
        fn clear(&mut self) {
            self.deref_mut().clear()
        }

        fn as_ptr(&self) -> *const u8 {
            self.deref().as_ptr()
        }

        fn as_mut_ptr(&mut self) -> *mut u8 {
            self.deref_mut().as_mut_ptr()
        }

        fn capacity(&self) -> usize {
            self.deref().capacity()
        }
    }
}

declare_impl! {
    (impl<T:  ReadBuf>  ReadBuf for &mut T),
    (impl<T: const ReadBuf> const ReadBuf for &mut T) {
        fn read(&mut self, len: usize) -> &[u8] {
            self.deref_mut().read(len)
        }

        unsafe fn get_continuous(&self, len: usize) -> &[u8] {
            self.deref().get_continuous(len)
        }

        fn remaining(&self) -> usize {
            self.deref().remaining()
        }

        fn advance(&mut self, len: usize) {
            self.deref_mut().advance(len)
        }

        fn pos(&self) -> usize {
            self.deref().pos()
        }

        unsafe fn set_pos(&mut self, pos: usize) {
            self.deref_mut().set_pos(pos)
        }
    }
}

declare_impl! {
    (impl<T:  WriteBuf>  WriteBuf for &mut T),
    (impl<T: const WriteBuf> const WriteBuf for &mut T) {
        fn write(&mut self, data: &[u8]) {
            self.deref_mut().write(data)
        }

        fn try_write(&mut self, data: &[u8]) -> Result<(), WriteBufferError> {
            self.deref_mut().try_write(data)
        }

        fn remaining_space(&self) -> usize {
            self.deref().remaining_space()
        }

        fn filled_pos(&self) -> usize {
            self.deref().filled_pos()
        }

        unsafe fn set_filled_pos(&mut self, filled_pos: usize) {
            self.deref_mut().set_filled_pos(filled_pos)
        }
    }
}
