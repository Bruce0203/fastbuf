use core::{alloc::Allocator, mem::MaybeUninit, ops::Range, slice::SliceIndex};

use crate::{declare_impl, declare_trait};

declare_trait! {
    pub trait Chunk<(T, const N: usize, A: Allocator)>: () {
        fn new_uninit_in(alloc: A) -> Self;
        fn new_uninit() -> Self;
        fn as_slice(&self) -> &[T; N];
        fn as_mut_slice(&mut self) -> &mut [T; N];
        fn as_ptr(&self) -> *const T;
        fn as_mut_ptr(&mut self) -> *mut T;
    }
}

declare_impl! {
    (impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for [T; N]),
    (impl<T, const N: usize, A: Allocator> const Chunk<T, N, A> for [T; N]) {
    fn as_slice(&self) -> &[T; N] {
        self
    }

    fn as_mut_slice(&mut self) -> &mut [T; N] {
        self
    }

    fn new_uninit_in(alloc: A) -> Self {
        core::mem::forget(alloc);
        <[T; N] as Chunk<T, N, A>>::new_uninit()
    }

    fn as_ptr(&self) -> *const T {
        <[T]>::as_ptr(self)
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        <[T]>::as_mut_ptr(self)
    }

    fn new_uninit() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_impl! {
    (impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for Box<[T; N], A>),
    (impl<T, const N: usize, A: Allocator> const Chunk<T, N, A> for Box<[T; N], A>) {
        default fn as_slice(&self) -> &[T; N] {
            self
        }

        default fn as_mut_slice(&mut self) -> &mut [T; N] {
            self
        }

        default fn new_uninit_in(alloc: A) -> Self {
            unsafe { Box::new_uninit_in(alloc).assume_init() }
        }

        default fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(&**self)
        }

        default fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(&mut **self)
        }

        default fn new_uninit() -> Self {
            unreachable!()
        }
    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_impl! {
    (impl<T, const N: usize> Chunk<T, N, std::alloc::Global> for Box<[T; N], std::alloc::Global>),
    (impl<T, const N: usize> const Chunk<T, N, std::alloc::Global> for Box<[T; N], std::alloc::Global>) {
        default fn as_slice(&self) -> &[T; N] {
            self
        }

        default fn as_mut_slice(&mut self) -> &mut [T; N] {
            self
        }

        default fn new_uninit_in(_alloc: std::alloc::Global) -> Self {
            unreachable!()
        }

        default fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(&**self)
        }

        default fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(&mut **self)
        }

        default fn new_uninit() -> Self {
            unsafe { Box::new_uninit().assume_init() }
        }
    }
}

