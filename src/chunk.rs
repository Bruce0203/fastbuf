use core::{alloc::Allocator, mem::MaybeUninit, ops::Range, slice::SliceIndex};

use crate::{declare_impl, declare_trait};

declare_trait! {
    pub trait Chunk<(T, const N: usize, A: Allocator)>: const (), (Clone, Copy) {
        fn new_in(alloc: A) -> Self;
        fn new() -> Self;
        fn as_slice(&self) -> &[T; N];
        fn as_mut_slice(&mut self) -> &mut [T; N];
        fn as_ptr(&self) -> *const T;
        fn as_mut_ptr(&mut self) -> *mut T;
    }
}

declare_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator> Chunk<T, N, A> for [T; N]),
    (impl<T: Copy + Clone, const N: usize, A: Allocator> const Chunk<T, N, A> for [T; N]) {
        #[inline(always)]
        fn as_slice(&self) -> &[T; N] {
            self
        }

        #[inline(always)]
        fn as_mut_slice(&mut self) -> &mut [T; N] {
            self
        }

        #[inline(always)]
        fn new_in(alloc: A) -> Self {
            core::mem::forget(alloc);
            <[T; N] as Chunk<T, N, A>>::new()
        }

        #[inline(always)]
        fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(self)
        }

        #[inline(always)]
        fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(self)
        }

        #[inline(always)]
        fn new() -> Self {
            unsafe { MaybeUninit::uninit().assume_init() }
        }
    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator + Clone> Chunk<T, N, A> for Box<[T; N], A>),
    (impl<T: Copy + Clone, const N: usize, A: Allocator + Clone> const Chunk<T, N, A> for Box<[T; N], A>) {
        #[inline(always)]
        default fn as_slice(&self) -> &[T; N] {
            self
        }

        #[inline(always)]
        default fn as_mut_slice(&mut self) -> &mut [T; N] {
            self
        }

        #[inline(always)]
        default fn new_uninit_in(alloc: A) -> Self {
            unsafe { Box::new_uninit_in(alloc).assume_init() }
        }

        #[inline(always)]
        default fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(&**self)
        }

        #[inline(always)]
        default fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(&mut **self)
        }

        #[inline(always)]
        default fn new_uninit() -> Self {
            unreachable!()
        }
    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_impl! {
    (impl<T: Copy + Clone, const N: usize> Chunk<T, N, std::alloc::Global> for Box<[T; N], std::alloc::Global>),
    (impl<T: Copy + Clone, const N: usize> const Chunk<T, N, std::alloc::Global> for Box<[T; N], std::alloc::Global>) {
        #[inline(always)]
        default fn as_slice(&self) -> &[T; N] {
            self
        }

        #[inline(always)]
        default fn as_mut_slice(&mut self) -> &mut [T; N] {
            self
        }

        #[inline(always)]
        default fn new_uninit_in(_alloc: std::alloc::Global) -> Self {
            unreachable!()
        }

        #[inline(always)]
        default fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(&**self)
        }

        #[inline(always)]
        default fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(&mut **self)
        }

        #[inline(always)]
        default fn new_uninit() -> Self {
            unsafe { Box::new_uninit().assume_init() }
        }
    }
}
