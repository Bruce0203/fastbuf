#![feature(min_specialization)]
#![feature(io_error_uncategorized)]

mod traits;

pub use traits::*;

mod buffer;
pub use buffer::*;

#[cfg(test)]
mod tests;
