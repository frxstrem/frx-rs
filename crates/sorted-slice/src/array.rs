use core::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use comparator::{Comparator, OrdComparator};

use super::{NotSortedError, SortedSlice};

/// An array that is known to be sorted.
pub struct SortedArray<T, const N: usize, C: Comparator<T> = OrdComparator> {
    _comparator: PhantomData<fn() -> C>,
    array: [T; N],
}

impl<T, const N: usize, C: Comparator<T>> SortedArray<T, N, C> {
    /// Create a sorted array from a regular array.
    ///
    /// # Safety
    ///
    /// The array must be sorted according to the comparator.
    pub const unsafe fn from_array_unchecked(array: [T; N]) -> Self {
        Self {
            _comparator: PhantomData,
            array,
        }
    }

    /// Create a sorted array from a regular array.
    ///
    /// # Errors
    ///
    /// If the array is not sorted, this will return an error with the original array.
    pub fn from_array(array: [T; N]) -> Result<SortedArray<T, N, C>, NotSortedError<[T; N]>> {
        if C::is_sorted(&array) {
            // SAFETY: The slice is sorted.
            unsafe { Ok(Self::from_array_unchecked(array)) }
        } else {
            Err(NotSortedError::new(array))
        }
    }
}

impl<T, const N: usize, C: Comparator<T>> Deref for SortedArray<T, N, C> {
    type Target = SortedSlice<T, C>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The array is known to be sorted.
        unsafe { SortedSlice::from_slice_unchecked(&self.array) }
    }
}

impl<T, const N: usize, C: Comparator<T>> DerefMut for SortedArray<T, N, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The array is known to be sorted.
        unsafe { SortedSlice::from_slice_mut_unchecked(&mut self.array) }
    }
}

impl<T, const N: usize, C: Comparator<T>> IntoIterator for SortedArray<T, N, C> {
    type Item = T;
    type IntoIter = core::array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter()
    }
}

impl<T, const N: usize, C: Comparator<T>> AsRef<[T]> for SortedArray<T, N, C> {
    fn as_ref(&self) -> &[T] {
        &self.array
    }
}

impl<T, const N: usize, C: Comparator<T>> Borrow<[T]> for SortedArray<T, N, C> {
    fn borrow(&self) -> &[T] {
        &self.array
    }
}

impl<T, const N: usize, C: Comparator<T>> AsRef<SortedSlice<T, C>> for SortedArray<T, N, C> {
    fn as_ref(&self) -> &SortedSlice<T, C> {
        self
    }
}

impl<T, const N: usize, C: Comparator<T>> Borrow<SortedSlice<T, C>> for SortedArray<T, N, C> {
    fn borrow(&self) -> &SortedSlice<T, C> {
        self
    }
}

impl<T, const N: usize, C: Comparator<T>> TryFrom<[T; N]> for SortedArray<T, N, C> {
    type Error = NotSortedError<[T; N]>;

    fn try_from(array: [T; N]) -> Result<Self, Self::Error> {
        Self::from_array(array)
    }
}

impl<T, const N: usize, C: Comparator<T>> From<SortedArray<T, N, C>> for [T; N] {
    fn from(sorted_array: SortedArray<T, N, C>) -> Self {
        sorted_array.array
    }
}

#[cfg(feature = "alloc")]
/// Sort an array and return a [`SortedArray`].
///
/// This will use a stable sort, which preserves the order of equal elements.
pub fn sort_array<T, const N: usize, C: Comparator<T>>(mut array: [T; N]) -> SortedArray<T, N, C> {
    C::sort(&mut array);

    // SAFETY: The slice is now sorted.
    unsafe { SortedArray::from_array_unchecked(array) }
}

/// Sort a slice and return a [`SortedArray`].
///
/// This may not preserve the order of equal elements. To do so, use
/// [`sort`](sort_array) instead.
pub fn sort_array_unstable<T, const N: usize, C: Comparator<T>>(
    mut array: [T; N],
) -> SortedArray<T, N, C> {
    C::sort_unstable(&mut array);

    // SAFETY: The slice is now sorted.
    unsafe { SortedArray::from_array_unchecked(array) }
}
