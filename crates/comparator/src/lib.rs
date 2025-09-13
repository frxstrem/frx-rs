//! Generic comparator trait.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, sync::Arc};

use core::cmp::{Ordering, Reverse};

/// Generic comparator between two values of the same type.
///
/// The default implementation should be `OrdComparator`, which uses the `Ord`
/// trait.
pub trait Comparator<T: ?Sized> {
    /// Compare two values.
    fn compare(left: &T, right: &T) -> Ordering;

    /// Check if a slice is sorted according to this comparator.
    fn is_sorted(slice: &[T]) -> bool
    where
        T: Sized,
    {
        slice.is_sorted_by(|left, right| Self::compare(left, right).is_le())
    }

    #[cfg(feature = "alloc")]
    /// Sort a slice using this comparator.
    ///
    /// This will use a stable sort, which preserves the order of equal elements.
    fn sort(slice: &mut [T])
    where
        T: Sized,
    {
        slice.sort_by(Self::compare)
    }

    /// Sort a slice using this comparator.
    ///
    /// This may not preserve the order of equal elements. To do so, use
    /// [`sort`](Comparator::sort) instead.
    fn sort_unstable(slice: &mut [T])
    where
        T: Sized,
    {
        slice.sort_unstable_by(Self::compare)
    }
}

/// Comparator using the `Ord` trait.
#[derive(Copy, Clone, Debug, Default)]
pub struct OrdComparator;

impl<T: ?Sized + Ord> Comparator<T> for OrdComparator {
    fn compare(left: &T, right: &T) -> Ordering {
        Ord::cmp(left, right)
    }
}

impl<C: Comparator<T>, T: ?Sized> Comparator<T> for Reverse<C> {
    fn compare(left: &T, right: &T) -> Ordering {
        C::compare(right, left)
    }
}

impl<C: Comparator<T>, T: ?Sized> Comparator<&T> for &C {
    fn compare(left: &&T, right: &&T) -> Ordering {
        C::compare(left, right)
    }
}

impl<C: Comparator<T>, T: ?Sized> Comparator<&mut T> for &mut C {
    fn compare(left: &&mut T, right: &&mut T) -> Ordering {
        C::compare(left, right)
    }
}

#[cfg(feature = "alloc")]
impl<C: Comparator<T>, T: ?Sized> Comparator<Box<T>> for Box<C> {
    fn compare(left: &Box<T>, right: &Box<T>) -> Ordering {
        C::compare(left, right)
    }
}

#[cfg(feature = "alloc")]
impl<C: Comparator<T>, T: ?Sized> Comparator<Rc<T>> for Rc<C> {
    fn compare(left: &Rc<T>, right: &Rc<T>) -> Ordering {
        C::compare(left, right)
    }
}

#[cfg(feature = "alloc")]
impl<C: Comparator<T>, T: ?Sized> Comparator<Arc<T>> for Arc<C> {
    fn compare(left: &Arc<T>, right: &Arc<T>) -> Ordering {
        C::compare(left, right)
    }
}
