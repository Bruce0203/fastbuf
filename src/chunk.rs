use core::{alloc::Allocator, mem::MaybeUninit};

pub trait Chunk<T, const N: usize, A: Allocator> {
    fn new_uninit_in(alloc: A) -> Self;
    fn as_slice(&self) -> &[T; N];
    fn as_mut_slice(&mut self) -> &mut [T; N];
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for [T; N] {
    fn as_slice(&self) -> &[T; N] {
        self
    }

    fn as_mut_slice(&mut self) -> &mut [T; N] {
        self
    }

    fn new_uninit_in(_alloc: A) -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    fn as_ptr(&self) -> *const T {
        <[T]>::as_ptr(self)
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        <[T]>::as_mut_ptr(self)
    }
}

impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for Box<[T; N], A> {
    fn as_slice(&self) -> &[T; N] {
        self
    }

    fn as_mut_slice(&mut self) -> &mut [T; N] {
        self
    }

    fn new_uninit_in(alloc: A) -> Self {
        unsafe { Box::new_uninit_in(alloc).assume_init() }
    }

    fn as_ptr(&self) -> *const T {
        <[T]>::as_ptr(&**self)
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        <[T]>::as_mut_ptr(&mut **self)
    }
}
