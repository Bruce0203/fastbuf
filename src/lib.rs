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

mod fast_slice;
