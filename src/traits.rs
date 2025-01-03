use crate::{declare_const_impl, declare_const_trait};
use core::ops::{Deref, DerefMut};
use std::alloc::Allocator;

declare_const_trait! {
    pub trait ChunkBuilder<(A: Allocator)>: const(), () {
        fn new_in(alloc: A) -> Self;
        fn new_zeroed() -> Self;
        fn new() -> Self;
    }
}

declare_const_trait! {
    pub trait Chunk<(T)>: const (), () {
        fn as_slice(&self) -> &[T];
        fn as_mut_slice(&mut self) -> &mut [T];
        fn as_ptr(&self) -> *const T;
        fn as_mut_ptr(&mut self) -> *mut T;
    }
}

declare_const_trait! {
    pub trait Buf<(T)>: const (ReadBuf<T>, WriteBuf<T>), () {
        fn clear(&mut self);
    }
}

declare_const_trait! {
    pub trait WriteBuf<(T)>: const (), (Chunk<T>) {
        fn write(&mut self, data: &[T]);
        fn try_write(&mut self, data: &[T]) -> Result<(), WriteBufferError>;
        fn try_write_fast<const LEN: usize>(&mut self, data: &[T; LEN]) -> Result<(), WriteBufferError>;
        fn remaining_space(&self) -> usize;
        fn filled_pos(&self) -> usize;
        unsafe fn set_filled_pos(&mut self, filled_pos: usize);
        fn capacity(&self) -> usize;
    }
}

declare_const_trait! {
    pub trait ReadBuf<(T)>: const (), (Chunk<T>) {
        fn read(&mut self, len: usize) -> &[T];
        unsafe fn get_continuous(&self, len: usize) -> &[T];
        unsafe fn get_continuous_mut(&mut self, len: usize) -> &mut [T];
        fn remaining(&self) -> usize;
        fn advance(&mut self, len: usize);
        fn pos(&self) -> usize;
        unsafe fn set_pos(&mut self, pos: usize);
    }
}

pub trait ReadToBuf<T> {
    #[cfg(feature = "const-trait")]
    fn read_to_buf(&mut self, buf: &mut impl const Buf<T>) -> Result<(), ()>;
    #[cfg(not(feature = "const-trait"))]
    fn read_to_buf(&mut self, buf: &mut impl Buf<T>) -> Result<(), ()>;
}

#[derive(Debug)]
pub enum WriteBufferError {
    BufferFull,
}

declare_const_impl! {
    (impl<T, S: Buf<T>> Buf<T> for &mut S),
    (impl<T, S: const Buf<T> + const Chunk<T>> const Buf<T> for &mut S) {
        fn clear(&mut self) {
            self.deref_mut().clear()
        }
    }
}

declare_const_impl! {
    (impl<T, S: ReadBuf<T>> ReadBuf<T> for &mut S),
    (impl<T, S: const ReadBuf<T> + const Chunk<T>> const ReadBuf<T> for &mut S) {
        fn read(&mut self, len: usize) -> &[T] {
            self.deref_mut().read(len)
        }

        unsafe fn get_continuous(&self, len: usize) -> &[T] {
            self.deref().get_continuous(len)
        }

        unsafe fn get_continuous_mut(&mut self, len: usize) -> &mut [T] {
            self.deref_mut().get_continuous_mut(len)
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

declare_const_impl! {
    (impl<T, S: WriteBuf<T>> WriteBuf<T> for &mut S),
    (impl<T, S: const WriteBuf<T> + const Chunk<T>> const WriteBuf<T> for &mut S) {
        fn write(&mut self, data: &[T]) {
            self.deref_mut().write(data)
        }

        fn try_write(&mut self, data: &[T]) -> Result<(), WriteBufferError> {
            self.deref_mut().try_write(data)
        }

        fn try_write_fast<const LEN: usize>(&mut self, data: &[T; LEN]) -> Result<(), WriteBufferError> {
            self.deref_mut().try_write_fast::<LEN>(data)
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

        fn capacity(&self) -> usize {
            self.deref().capacity()
        }
    }
}

declare_const_impl! {
    (impl<T, S: Chunk<T>> Chunk<T> for &mut S),
    (impl<T, S: const Chunk<T>> const Chunk<T> for &mut S) {
        fn as_slice(&self) ->  &[T] {
            self.deref().as_slice()
        }

        fn as_mut_slice(&mut self) ->  &mut [T] {
            self.deref_mut().as_mut_slice()
        }

        fn as_ptr(&self) ->  *const T {
            self.deref().as_ptr()
        }

        fn as_mut_ptr(&mut self) ->  *mut T {
            self.deref_mut().as_mut_ptr()
        }
    }
}
