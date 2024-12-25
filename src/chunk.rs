use core::{
    alloc::Allocator,
    mem::{transmute, transmute_copy, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};

use crate::{declare_impl, declare_trait};

declare_trait! {
    pub trait Chunk<(T, const N: usize, A: Allocator)>: () {
        fn new_uninit_in(alloc: A) -> Self;
        fn new_uninit() -> Self;
        fn as_slice(&self) -> &[T];
        fn as_mut_slice(&mut self) -> &mut [T];
        fn as_ptr(&self) -> *const T;
        fn as_mut_ptr(&mut self) -> *mut T;
    }
}

declare_impl! {
    (impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for [T; N]),
    (impl<T, const N: usize, A: Allocator> const Chunk<T, N, A> for [T; N]) {
        #[inline(always)]
        fn as_slice(&self) -> &[T] {
            self
        }

        #[inline(always)]
        fn as_mut_slice(&mut self) -> &mut [T] {
            self
        }

        #[inline(always)]
        fn new_uninit_in(alloc: A) -> Self {
            core::mem::forget(alloc);
            <[T; N] as Chunk<T, N, A>>::new_uninit()
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
        fn new_uninit() -> Self {
            unsafe { MaybeUninit::uninit().assume_init() }
        }
    }
}

fn asdf() {
    let a: *const u8 = &0;
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_impl! {
    (impl<T, const N: usize, A: Allocator> Chunk<T, N, A> for Box<[T; N], A>),
    (impl<T, const N: usize, A: Allocator> const Chunk<T, N, A> for Box<[T; N], A>) {
        #[inline(always)]
        default fn as_slice(&self) -> &[T] {
            &**self
        }

        #[inline(always)]
        default fn as_mut_slice(&mut self) -> &mut [T] {
            &mut **self
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
    (impl<T, const N: usize> Chunk<T, N, std::alloc::Global> for Box<[T; N], std::alloc::Global>),
    (impl<T, const N: usize> const Chunk<T, N, std::alloc::Global> for Box<[T; N], std::alloc::Global>) {
        #[inline(always)]
        default fn as_slice(&self) -> &[T] {
            self.deref()
        }

        #[inline(always)]
        default fn as_mut_slice(&mut self) -> &mut [T] {
            self.deref_mut()
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
