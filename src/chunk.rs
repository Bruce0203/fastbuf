use core::{alloc::Allocator, mem::MaybeUninit};

use crate::{declare_const_impl, Chunk, ChunkBuilder};

declare_const_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator> ChunkBuilder<A> for [T; N]),
    (impl<T: Copy + Clone, const N: usize, A: Allocator> const ChunkBuilder<A> for [T; N]) {
        #[inline(always)]
        fn new_in(alloc: A) -> Self {
            core::mem::forget(alloc);
            <[T; N] as ChunkBuilder<A>>::new()
        }

        #[inline(always)]
        fn new() -> Self {
            unsafe { MaybeUninit::uninit().assume_init() }
        }

        #[inline(always)]
        fn new_zeroed() -> Self {
            unsafe { MaybeUninit::zeroed().assume_init() }
        }
    }
}

declare_const_impl! {
    (impl<T: Copy + Clone, const N: usize> Chunk<T> for [T; N]),
    (impl<T: Copy + Clone, const N: usize> const Chunk<T> for [T; N]) {
        #[inline(always)]
        fn as_slice(&self) -> &[T] {
            self
        }

        #[inline(always)]
        fn as_mut_slice(&mut self) -> &mut [T] {
            self
        }

        #[inline(always)]
        fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(self)
        }

        #[inline(always)]
        fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(self)
        }
    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_const_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator + Copy + Clone> ChunkBuilder<A> for Box<[T; N], A>),
    (impl<T: Copy + Clone, const N: usize, A: Allocator + Copy + Clone> const ChunkBuilder<A> for Box<[T; N], A>) {
        #[inline(always)]
        default fn new_in(alloc: A) -> Self {
            unsafe { Box::new_uninit_in(alloc).assume_init() }
        }

        #[inline(always)]
        default fn new() -> Self {
            unreachable!()
        }

        #[inline(always)]
        default fn new_zeroed() -> Self {
            unreachable!()
        }
    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_const_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator + Copy + Clone> Chunk<T> for Box<[T; N], A>),
    (impl<T: Copy + Clone, const N: usize, A: Allocator + Copy + Clone> const Chunk<T> for Box<[T; N], A>) {
        #[inline(always)]
        default fn as_slice(&self) -> &[T] {
            &**self
        }

        #[inline(always)]
        default fn as_mut_slice(&mut self) -> &mut [T] {
            &mut **self
        }

        #[inline(always)]
        default fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(&**self)
        }

        #[inline(always)]
        default fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(&mut **self)
        }
    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_const_impl! {
    (impl<T: Copy + Clone, const N: usize> ChunkBuilder<std::alloc::Global> for Box<[T; N], std::alloc::Global>),
    (impl<T: Copy + Clone, const N: usize> const ChunkBuilder<std::alloc::Global> for Box<[T; N], std::alloc::Global>) {
        #[inline(always)]
        default fn new_in(_alloc: std::alloc::Global) -> Self {
            unreachable!()
        }

        #[inline(always)]
        default fn new() -> Self {
            unsafe { Box::new_uninit().assume_init() }
        }

        #[inline(always)]
        default fn new_zeroed() -> Self {
            unsafe { Box::new_zeroed().assume_init() }
        }
    }
}

#[cfg(all(not(feature = "const-trait"), feature = "std"))]
declare_const_impl! {
    (impl<T: Copy + Clone, const N: usize> Chunk<T> for Box<[T; N], std::alloc::Global>),
    (impl<T: Copy + Clone, const N: usize> const Chunk<T> for Box<[T; N], std::alloc::Global>) {
        #[inline(always)]
        default fn as_slice(&self) -> &[T] {
            &**self
        }

        #[inline(always)]
        default fn as_mut_slice(&mut self) -> &mut [T] {
            &mut **self
        }

        #[inline(always)]
        default fn as_ptr(&self) -> *const T {
            <[T]>::as_ptr(&**self)
        }

        #[inline(always)]
        default fn as_mut_ptr(&mut self) -> *mut T {
            <[T]>::as_mut_ptr(&mut **self)
        }
    }
}
