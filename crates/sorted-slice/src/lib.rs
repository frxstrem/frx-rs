//! Utilities for working with sorted slices and arrays.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod array;
mod slice;
#[cfg(feature = "alloc")]
mod vec;

pub use self::array::*;
pub use self::slice::*;
#[cfg(feature = "alloc")]
pub use self::vec::*;

/// Error when attempting to convert an unsorted slice or array into a [`SortedSlice`] or
/// [`SortedArray`].
#[derive(Debug)]
pub struct NotSortedError<T = ()> {
    inner: T,
}

impl<T> NotSortedError<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Get the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl core::fmt::Display for NotSortedError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("slice is not sorted")
    }
}

impl core::error::Error for NotSortedError {
    fn description(&self) -> &str {
        "slice is not sorted"
    }
}
