use core::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Bound, Deref, Index, RangeBounds},
};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use comparator::{Comparator, OrdComparator};

use super::NotSortedError;

/// A slice that is known to be sorted.
#[repr(transparent)]
pub struct SortedSlice<T, C: Comparator<T> = OrdComparator> {
    _comparator: PhantomData<fn() -> C>,
    slice: [T],
}

impl<T, C: Comparator<T>> SortedSlice<T, C> {
    /// Create a sorted slice from a regular slice.
    ///
    /// # Safety
    ///
    /// The slice must be sorted according to the comparator.
    pub const unsafe fn from_slice_unchecked(slice: &[T]) -> &SortedSlice<T, C> {
        // SAFETY: [T] and SortedSlice<T, C> have the same layout.
        unsafe { &*(slice as *const [T] as *const SortedSlice<T, C>) }
    }

    /// Create a sorted slice from a mutable slice.
    ///
    /// # Safety
    ///
    /// The slice must be sorted according to the comparator.
    pub const unsafe fn from_slice_mut_unchecked(slice: &mut [T]) -> &mut SortedSlice<T, C> {
        // SAFETY: [T] and SortedSlice<T, C> have the same layout, and SortedSlice<T, C> has no
        // custom destructor.
        unsafe { &mut *(slice as *mut [T] as *mut SortedSlice<T, C>) }
    }

    #[cfg(feature = "alloc")]
    /// Create a sorted slice from a boxed slice.
    ///
    /// # Safety
    ///
    /// The slice must be sorted according to the comparator.
    pub unsafe fn from_boxed_slice_unchecked(slice: Box<[T]>) -> Box<SortedSlice<T, C>> {
        // SAFETY: [T] and SortedSlice<T, C> have the same layout, and SortedSlice<T, C> has no
        // custom destructor.
        unsafe { Box::from_raw(Box::into_raw(slice) as *mut SortedSlice<T, C>) }
    }

    /// Create a sorted slice from a regular slice.
    ///
    /// # Errors
    ///
    /// If the slice is not sorted, this will return an error.
    pub fn from_slice(slice: &[T]) -> Result<&SortedSlice<T, C>, NotSortedError> {
        if C::is_sorted(slice) {
            // SAFETY: The slice is sorted.
            Ok(unsafe { Self::from_slice_unchecked(slice) })
        } else {
            Err(NotSortedError::new(()))
        }
    }

    /// Create a sorted slice from a mutable slice.
    ///
    /// # Errors
    ///
    /// If the slice is not sorted, this will return an error.
    pub fn from_slice_mut(slice: &mut [T]) -> Result<&mut SortedSlice<T, C>, NotSortedError> {
        if C::is_sorted(slice) {
            // SAFETY: The slice is sorted.
            Ok(unsafe { Self::from_slice_mut_unchecked(slice) })
        } else {
            Err(NotSortedError::new(()))
        }
    }

    #[cfg(feature = "alloc")]
    /// Create a sorted slice from a boxed slice.
    ///
    /// # Errors
    ///
    /// If the slice is not sorted, this will return an error with the original boxed slice.
    pub fn from_boxed_slice_mut(
        slice: Box<[T]>,
    ) -> Result<Box<SortedSlice<T, C>>, NotSortedError<Box<[T]>>> {
        if C::is_sorted(&slice) {
            // SAFETY: The slice is sorted.
            Ok(unsafe { Self::from_boxed_slice_unchecked(slice) })
        } else {
            Err(NotSortedError::new(slice))
        }
    }

    /// Access the inner slice.
    pub const fn as_slice(&self) -> &[T] {
        &self.slice
    }

    /// Access the inner slice with a mutable reference.
    ///
    /// # Safety
    ///
    /// The elements of the slice must only be changed in ways that preserve
    /// the order according to the comparator.
    pub unsafe fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.slice
    }

    /// Index the slice.
    pub fn get<I: SortedSliceIndex<T, C>>(&self, index: I) -> Option<&I::Output> {
        index.get(self)
    }

    /// Index the slice without checking bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index is in bounds.
    pub unsafe fn get_unchecked<I: SortedSliceIndex<T, C>>(&self, index: I) -> &I::Output {
        // SAFETY: The caller ensures that the index is in bounds.
        unsafe { index.get_unchecked(self) }
    }

    /// Binary search for an element in the slice.
    ///
    /// # Returns
    ///
    /// If the element is found, this returns `Ok(index)`, where `index` is the index of the element.
    /// If the element is not found, this returns `Err(index)`, where `index` is the index where the element should be inserted.
    #[allow(clippy::missing_errors_doc)]
    pub fn binary_search(&self, key: &T) -> Result<usize, usize> {
        self.slice.binary_search_by(|el| C::compare(el, key))
    }
}

impl<T, C: Comparator<T>> Deref for SortedSlice<T, C> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.slice
    }
}

impl<T, C: Comparator<T>, I: SortedSliceIndex<T, C>> Index<I> for SortedSlice<T, C> {
    type Output = I::Output;

    fn index(&self, index: I) -> &I::Output {
        index.get(self).expect("index out of bounds")
    }
}

impl<'a, T, C: Comparator<T>> IntoIterator for &'a SortedSlice<T, C> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.slice.iter()
    }
}

impl<T, C: Comparator<T>> AsRef<[T]> for SortedSlice<T, C> {
    fn as_ref(&self) -> &[T] {
        &self.slice
    }
}

impl<T, C: Comparator<T>> Borrow<[T]> for SortedSlice<T, C> {
    fn borrow(&self) -> &[T] {
        &self.slice
    }
}

impl<'a, T, C: Comparator<T>> TryFrom<&'a [T]> for &'a SortedSlice<T, C> {
    type Error = NotSortedError;

    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        SortedSlice::from_slice(value)
    }
}

impl<'a, T, C: Comparator<T>> TryFrom<&'a mut [T]> for &'a mut SortedSlice<T, C> {
    type Error = NotSortedError;

    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        SortedSlice::from_slice_mut(value)
    }
}

#[cfg(feature = "alloc")]
impl<T, C: Comparator<T>> TryFrom<Box<[T]>> for Box<SortedSlice<T, C>> {
    type Error = NotSortedError<Box<[T]>>;

    fn try_from(value: Box<[T]>) -> Result<Self, Self::Error> {
        SortedSlice::from_boxed_slice_mut(value)
    }
}

#[cfg(feature = "alloc")]
impl<T, C: Comparator<T>> From<Box<SortedSlice<T, C>>> for Box<[T]> {
    fn from(value: Box<SortedSlice<T, C>>) -> Self {
        // SAFETY: `SortedSlice` is a transparent wrapper around `[T]`, so casting
        // is safe.
        unsafe { Box::from_raw(Box::into_raw(value) as *mut [T]) }
    }
}

#[cfg(feature = "alloc")]
/// Sort a slice and return a [`SortedSlice`].
///
/// This will use a stable sort, which preserves the order of equal elements.
pub fn sort_slice<T, C: Comparator<T>>(slice: &mut [T]) -> &mut SortedSlice<T, C> {
    C::sort(slice);

    // SAFETY: The slice is now sorted.
    unsafe { SortedSlice::from_slice_mut_unchecked(slice) }
}

/// Sort a slice and return a [`SortedSlice`].
///
/// This may not preserve the order of equal elements. To do so, use
/// [`sort`](sort_slice) instead.
pub fn sort_slice_unstable<T, C: Comparator<T>>(slice: &mut [T]) -> &mut SortedSlice<T, C> {
    C::sort_unstable(slice);

    // SAFETY: The slice is now sorted.
    unsafe { SortedSlice::from_slice_mut_unchecked(slice) }
}

/// A type that can be used to index a [`SortedSlice`].
pub trait SortedSliceIndex<T, C: Comparator<T>> {
    /// The output type when indexing the slice.
    type Output: ?Sized;

    /// Get a reference to the output type, if the index is in bounds.
    fn get(self, slice: &SortedSlice<T, C>) -> Option<&Self::Output>;

    /// Get a reference to the output type without checking bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index is in bounds.
    unsafe fn get_unchecked(self, slice: &SortedSlice<T, C>) -> &Self::Output;
}

impl<T, C: Comparator<T>> SortedSliceIndex<T, C> for usize {
    type Output = T;

    fn get(self, slice: &SortedSlice<T, C>) -> Option<&Self::Output> {
        slice.slice.get(self)
    }

    unsafe fn get_unchecked(self, slice: &SortedSlice<T, C>) -> &Self::Output {
        // SAFETY: The caller ensures that the index is in bounds.
        unsafe { slice.slice.get_unchecked(self) }
    }
}

macro_rules! slice_index_range {
    ($ty:ty) => {
        impl<T, C: Comparator<T>> SortedSliceIndex<T, C> for $ty {
            type Output = SortedSlice<T, C>;

            fn get(self, slice: &SortedSlice<T, C>) -> Option<&Self::Output> {
                let subslice = slice.slice.get(self)?;

                // SAFETY: The subslice is already sorted.
                unsafe { Some(SortedSlice::from_slice_unchecked(subslice)) }
            }

            unsafe fn get_unchecked(self, slice: &SortedSlice<T, C>) -> &Self::Output {
                // SAFETY: The caller ensures that the index is in bounds.
                unsafe {
                    let subslice = slice.slice.get_unchecked(self);
                    SortedSlice::from_slice_unchecked(subslice)
                }
            }
        }
    };
}

slice_index_range!(core::ops::Range<usize>);
slice_index_range!(core::ops::RangeFrom<usize>);
slice_index_range!(core::ops::RangeTo<usize>);
slice_index_range!(core::ops::RangeInclusive<usize>);
slice_index_range!(core::ops::RangeToInclusive<usize>);
slice_index_range!(core::ops::RangeFull);
slice_index_range!((core::ops::Bound<usize>, core::ops::Bound<usize>));

/// A type that can be used to index a [`SortedSlice`] with a search key.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Search<T: ?Sized>(pub T);

macro_rules! search_index_range {
    ($T:ident => $ty:ty) => {
        impl<$T, C: Comparator<T>> SortedSliceIndex<$T, C> for $ty {
            type Output = SortedSlice<$T, C>;

            fn get(self, slice: &SortedSlice<$T, C>) -> Option<&Self::Output> {
                let start = binary_search_bound(slice, self.start_bound(), |key| key);

                let end = match self.end_bound() {
                    Bound::Included(Search(key)) => match slice.binary_search(key) {
                        Ok(index) => Bound::Included(index),
                        Err(index) => Bound::Excluded(index),
                    },
                    Bound::Excluded(Search(key)) => match slice.binary_search(key) {
                        Ok(index) | Err(index) => Bound::Excluded(index),
                    },
                    Bound::Unbounded => Bound::Unbounded,
                };

                let subslice = slice.get((start, end))?;

                // SAFETY: The subslice is already sorted.
                unsafe { Some(SortedSlice::from_slice_unchecked(subslice)) }
            }

            unsafe fn get_unchecked(self, slice: &SortedSlice<$T, C>) -> &Self::Output {
                let start = binary_search_bound(slice, self.start_bound(), |key| key);

                let end = match self.end_bound() {
                    Bound::Included(Search(key)) => match slice.binary_search(key) {
                        Ok(index) => Bound::Included(index),
                        Err(index) => Bound::Excluded(index),
                    },
                    Bound::Excluded(Search(key)) => match slice.binary_search(key) {
                        Ok(index) | Err(index) => Bound::Excluded(index),
                    },
                    Bound::Unbounded => Bound::Unbounded,
                };

                // SAFETY: The caller ensures that the index is in bounds.
                unsafe {
                    let subslice = slice.get_unchecked((start, end));
                    SortedSlice::from_slice_unchecked(subslice)
                }
            }
        }
    };
}

search_index_range!(T => core::ops::Range<Search<T>>);
search_index_range!(T => core::ops::RangeFrom<Search<T>>);
search_index_range!(T => core::ops::RangeTo<Search<T>>);
search_index_range!(T => core::ops::RangeInclusive<Search<T>>);
search_index_range!(T => core::ops::RangeToInclusive<Search<T>>);
search_index_range!(T => (core::ops::Bound<Search<T>>, core::ops::Bound<Search<T>>));

search_index_range!(T => core::ops::Range<Search<&T>>);
search_index_range!(T => core::ops::RangeFrom<Search<&T>>);
search_index_range!(T => core::ops::RangeTo<Search<&T>>);
search_index_range!(T => core::ops::RangeInclusive<Search<&T>>);
search_index_range!(T => core::ops::RangeToInclusive<Search<&T>>);
search_index_range!(T => (core::ops::Bound<Search<&T>>, core::ops::Bound<Search<&T>>));

fn binary_search_bound<T, C: Comparator<T>, Q>(
    slice: &SortedSlice<T, C>,
    bound: Bound<&Search<Q>>,
    map_fn: impl FnOnce(&Q) -> &T,
) -> Bound<usize> {
    match bound {
        Bound::Included(Search(key)) => match slice.binary_search(map_fn(key)) {
            Ok(index) => Bound::Included(index),
            Err(index) => Bound::Excluded(index),
        },
        Bound::Excluded(Search(key)) => match slice.binary_search(map_fn(key)) {
            Ok(index) | Err(index) => Bound::Excluded(index),
        },
        Bound::Unbounded => Bound::Unbounded,
    }
}
