use core::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use alloc::vec::{self, Vec};

use comparator::{Comparator, OrdComparator};

use crate::{NotSortedError, SortedSlice};

/// A vector that is known to be sorted.
pub struct SortedVec<T, C: Comparator<T> = OrdComparator> {
    _comparator: PhantomData<fn() -> C>,
    vec: Vec<T>,
}

impl<T, C: Comparator<T>> SortedVec<T, C> {
    /// Create a sorted vector from a regular vector.
    ///
    /// # Safety
    ///
    /// The vector must be sorted according to the comparator.
    pub const unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self {
            _comparator: PhantomData,
            vec,
        }
    }

    /// Create a sorted vector from a regular vector.
    ///
    /// # Errors
    ///
    /// If the vector is not sorted, this will return an error with the original vector.
    pub fn from_vec(vec: Vec<T>) -> Result<SortedVec<T, C>, NotSortedError<Vec<T>>> {
        if C::is_sorted(&vec) {
            // SAFETY: The slice is sorted.
            unsafe { Ok(Self::from_vec_unchecked(vec)) }
        } else {
            Err(NotSortedError::new(vec))
        }
    }
}

impl<T, C: Comparator<T>> Deref for SortedVec<T, C> {
    type Target = SortedSlice<T, C>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The vector is known to be sorted.
        unsafe { SortedSlice::from_slice_unchecked(&self.vec) }
    }
}

impl<T, C: Comparator<T>> DerefMut for SortedVec<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The vector is known to be sorted.
        unsafe { SortedSlice::from_slice_mut_unchecked(&mut self.vec) }
    }
}

impl<T, C: Comparator<T>> IntoIterator for SortedVec<T, C> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T, C: Comparator<T>> AsRef<[T]> for SortedVec<T, C> {
    fn as_ref(&self) -> &[T] {
        &self.vec
    }
}

impl<T, C: Comparator<T>> Borrow<[T]> for SortedVec<T, C> {
    fn borrow(&self) -> &[T] {
        &self.vec
    }
}

impl<T, C: Comparator<T>> AsRef<SortedSlice<T, C>> for SortedVec<T, C> {
    fn as_ref(&self) -> &SortedSlice<T, C> {
        self
    }
}

impl<T, C: Comparator<T>> Borrow<SortedSlice<T, C>> for SortedVec<T, C> {
    fn borrow(&self) -> &SortedSlice<T, C> {
        self
    }
}

impl<T, C: Comparator<T>> TryFrom<Vec<T>> for SortedVec<T, C> {
    type Error = NotSortedError<Vec<T>>;

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        Self::from_vec(vec)
    }
}

impl<T, C: Comparator<T>> From<SortedVec<T, C>> for Vec<T> {
    fn from(sorted_vec: SortedVec<T, C>) -> Vec<T> {
        sorted_vec.vec
    }
}

/// Sort a vector and return a [`SortedVec`].
///
/// This will use a stable sort, which preserves the order of equal elements.
pub fn sort_vec<T, C: Comparator<T>>(mut vec: Vec<T>) -> SortedVec<T, C> {
    C::sort(&mut vec);

    // SAFETY: The vector is sorted.
    unsafe { SortedVec::from_vec_unchecked(vec) }
}

/// Sort a vector and return a [`SortedVec`].
///
/// This may not preserve the order of equal elements. To do so, use
/// [`sort_vec`] instead.
pub fn sort_vec_unstable<T, C: Comparator<T>>(mut vec: Vec<T>) -> SortedVec<T, C> {
    C::sort_unstable(&mut vec);

    // SAFETY: The vector is sorted.
    unsafe { SortedVec::from_vec_unchecked(vec) }
}
