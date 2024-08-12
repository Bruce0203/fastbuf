mod traits;
use std::io::BorrowedBuf;

pub use traits::*;

mod buffer;
pub use buffer::*;

#[cfg(test)]
mod tests;

fn main2() {
    BorrowedBuf::from(value)
}
