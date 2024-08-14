use std::ops::{Deref, DerefMut};

pub trait Buf {
    fn clear(&mut self);
    fn pos(&self) -> usize;
    fn filled_pos(&self) -> usize;
    unsafe fn set_filled_pos(&mut self, value: usize);
    unsafe fn set_pos(&mut self, value: usize);
}

pub trait WriteBuf: Buf {
    fn write(&mut self, data: &[u8]);
    fn try_write(&mut self, data: &[u8]) -> Result<(), ()>;
    fn remaining_space(&self) -> usize;
}

pub trait ReadBuf: Buf {
    fn read(&mut self, len: usize) -> &[u8];
    fn advance(&mut self, len: usize);
    fn get_continuous(&self, len: usize) -> &[u8];
    fn remaining(&self) -> usize;
}

impl<T: Buf> Buf for Box<T> {
    fn clear(&mut self) {
        self.deref_mut().clear()
    }

    fn pos(&self) -> usize {
        self.deref().pos()
    }

    fn filled_pos(&self) -> usize {
        self.deref().filled_pos()
    }

    unsafe fn set_filled_pos(&mut self, value: usize) {
        self.deref_mut().set_filled_pos(value)
    }

    unsafe fn set_pos(&mut self, value: usize) {
        self.deref_mut().set_pos(value)
    }
}

impl<T: ReadBuf> ReadBuf for Box<T> {
    fn read(&mut self, len: usize) -> &[u8] {
        self.deref_mut().read(len)
    }

    fn advance(&mut self, len: usize) {
        self.deref_mut().advance(len)
    }

    fn get_continuous(&self, len: usize) -> &[u8] {
        self.deref().get_continuous(len)
    }

    fn remaining(&self) -> usize {
        self.deref().remaining()
    }
}

impl<T: WriteBuf> WriteBuf for Box<T> {
    fn write(&mut self, data: &[u8]) {
        self.deref_mut().write(data)
    }

    fn try_write(&mut self, data: &[u8]) -> Result<(), ()> {
        self.deref_mut().try_write(data)
    }

    fn remaining_space(&self) -> usize {
        self.deref().remaining_space()
    }
}
