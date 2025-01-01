#![feature(negative_impls)]
#![feature(auto_traits)]
#![feature(maybe_uninit_uninit_array)]
#![feature(slice_index_methods)]
#![feature(min_specialization)]
#![feature(const_copy_from_slice)]
#![feature(const_trait_impl)]
#![cfg_attr(feature = "std", feature(new_zeroed_alloc))]
#![feature(allocator_api)]
#![cfg_attr(test, feature(test))]
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

#[cfg(test)]
extern crate self as fastbuf;
#[cfg(test)]
extern crate test;

#[cfg(not(feature = "std"))]
extern crate core as std;

mod traits;
pub use traits::*;

mod buffer;
pub use buffer::*;

mod chunk;

#[cfg(not(feature = "std"))]
pub(crate) struct EmptyAlloc;

#[cfg(not(feature = "std"))]
unsafe impl std::alloc::Allocator for EmptyAlloc {
    fn allocate(
        &self,
        _layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unreachable!()
    }

    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        unreachable!()
    }
}

pub(crate) mod macros {

    #[macro_export]
    macro_rules! declare_const_fn {
        ($(#[$($attrs:tt)*])* $visibility:vis fn $($tokens:tt)*) => {
            #[cfg(feature = "const-trait")]
            $(#[$($attrs)*])*
            $visibility const fn $($tokens)*

            #[cfg(not(feature = "const-trait"))]
            $visibility fn $($tokens)*
        };
    }

    #[macro_export]
    macro_rules! declare_const_trait {
        ($visibility:vis trait $name:ident<($($generics:tt)*)>
         : const ($($const_supertrait:path),*), ($($supertrait:path),*) {$($body:tt)*}) => {
            #[cfg(not(feature = "const-trait"))]
            $visibility trait $name<$($generics)*>: $($const_supertrait +)* $($supertrait + )* {
                $($body)*
            }

            #[cfg(feature = "const-trait")]
            #[const_trait]
            $visibility trait $name<$($generics)*>: $(const $const_supertrait +)* $($supertrait + )* {
                $($body)*
            }
        };
    }

    #[macro_export]
    macro_rules! declare_const_impl {
        (($($impl:tt)*), ($($impl_const:tt)*) {$($body:tt)*}) => {
            #[cfg(feature = "const-trait")]
            $($impl_const)* { $($body)* }

            #[cfg(not(feature = "const-trait"))]
            $($impl)* { $($body)* }
        };
    }

    #[cfg(feature = "const-trait")]
    #[macro_export]
    macro_rules! const_min {
        ($a:expr, $b:expr) => {
            if $a > $b {
                $b
            } else {
                $a
            }
        };
    }

    #[cfg(not(feature = "const-trait"))]
    #[macro_export]
    macro_rules! const_min {
        ($a:expr, $b:expr) => {
            core::cmp::min($a, $b)
        };
    }
}
fn asdf() {
    let v = (&0_u8) as *const u8;
    
}

/// Copies `N` or `n` bytes from `src` to `dst` depending on if `src` lies within a memory page.
/// https://stackoverflow.com/questions/37800739/is-it-safe-to-read-past-the-end-of-a-buffer-within-the-same-page-on-x86-and-x64
/// # Safety
/// Same as [`std::ptr::copy_nonoverlapping`] but with the additional requirements that
/// `n != 0 && n <= N` and `dst` has room for a `[T; N]`.
/// Is a macro instead of an `#[inline(always)] fn` because it optimizes better.
macro_rules! unsafe_wild_copy {
    // pub unsafe fn wild_copy<T, const N: usize>(src: *const T, dst: *mut T, n: usize) {
    ([$T:ident; $N:ident], $src:ident, $dst:ident, $n:ident) => {
        debug_assert!($n != 0 && $n <= $N);

        let page_size = 4096;
        let read_size = core::mem::size_of::<[$T; $N]>();
        let src_ptr_as_usize = $src.byte_offset_from($src) as usize;
        let within_page = src_ptr_as_usize & (page_size - 1) < (page_size - read_size) && cfg!(all(
            // Miri doesn't like this.
            not(miri),
            // cargo fuzz's memory sanitizer complains about buffer overrun.
            // Without nightly we can't detect memory sanitizers, so we check debug_assertions.
            not(debug_assertions),
            // x86/x86_64/aarch64 all have min page size of 4096, so reading past the end of a non-empty
            // buffer won't page fault.
            any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
        ));

        if within_page {
            *($dst as *mut core::mem::MaybeUninit<[$T; $N]>) = core::ptr::read($src as *const core::mem::MaybeUninit<[$T; $N]>);
        } else {
            $src.copy_to_nonoverlapping($dst, $n);
        }
    }
}
pub(crate) use unsafe_wild_copy;
