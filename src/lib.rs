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
pub use chunk::*;

pub struct EmptyAlloc;
unsafe impl std::alloc::Allocator for EmptyAlloc {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unreachable!()
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        unreachable!()
    }
}

pub(crate) mod macros {

    #[macro_export]
    macro_rules! declare_trait {
        ($visibility:vis trait $name:ident<($($generics:tt)*)>: ($($supertrait:path),*) {$($body:tt)*}) => {
            #[cfg(not(feature = "const-trait"))]
            $visibility trait $name<$($generics)*>: $($supertrait + )* {
                $($body)*
            }

            #[cfg(feature = "const-trait")]
            #[const_trait]
            $visibility trait $name<$($generics)*>: $(const $supertrait +)* {
                $($body)*
            }
        };
    }

    #[macro_export]
    macro_rules! declare_impl {
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
            konst::min!($a, $b)
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
