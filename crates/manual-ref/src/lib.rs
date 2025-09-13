//! Unsafe references without lifetimes.

#![no_std]

use core::{
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::NonNull,
};

/// An immutable reference to a value of type `T`, without a lifetime.
pub struct ManualRef<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T: ?Sized> ManualRef<T> {
    /// Create a new `ManualRef` from a reference.
    ///
    /// # Safety
    ///
    /// Erasing the lifetime of a reference is inherently safe. Because of this,
    /// this method is unsafe and the caller must ensure that Rust's reference
    /// lifetime rules are not violated.
    ///
    /// Among other things, this means that while the `ManualRef` exists, the data pointed
    /// to by `ptr` must not be moved or deallocated, and there may not be any mutable
    /// references to the data.
    pub const unsafe fn new(ptr: &T) -> ManualRef<T> {
        // SAFETY: `NonNull::new_unchecked` is safe to call with a pointer casted from a reference.
        // We do this because the function is a const function.
        unsafe {
            ManualRef {
                ptr: NonNull::new_unchecked(ptr as *const _ as *mut _),
            }
        }
    }

    /// Create a new `ManualRef` from a static reference.
    ///
    /// This is safe because the reference will never be invalidated.
    pub const fn from_static(ptr: &'static T) -> ManualRef<T> {
        // SAFETY: `ptr` has a static lifetime so it will always outlive the `ManualRef`.
        unsafe { Self::new(ptr) }
    }

    /// Create a new `ManualRef` from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and aligned and point to a valid value of `T`.
    /// In addition, the same safety requirements as [`ManualRef::new`] applies.
    pub const unsafe fn from_ptr(ptr: *const T) -> ManualRef<T> {
        // SAFETY: caller ensures that `ptr` is non-null.
        unsafe {
            ManualRef {
                ptr: NonNull::new_unchecked(ptr as *mut _),
            }
        }
    }

    /// Create a new `ManualRef` from a pinned reference.
    ///
    /// # Safety
    ///
    /// This method has the same safety requirements as [`ManualRef::new`].
    pub const unsafe fn new_pinned(ptr: Pin<&T>) -> Pin<ManualRef<T>> {
        // SAFETY: `ManualRef<T>` preserves pinnability of `&T` and the caller ensures that
        // the safety requirements of `ManualRef::new` are met.
        unsafe { Pin::new_unchecked(ManualRef::new(Pin::into_inner_unchecked(ptr))) }
    }

    /// Convert to a raw pointer.
    pub const fn as_ptr(this: &Self) -> *const T {
        this.ptr.as_ptr().cast_const()
    }
}

// SAFETY: `ManualRef` has the same thread safety invariants as `&T`.
unsafe impl<T: ?Sized + Sync> Send for ManualRef<T> {}

// SAFETY: `ManualRef` has the same thread safety invariants as `&T`.
unsafe impl<T: ?Sized + Sync> Sync for ManualRef<T> {}

impl<T: ?Sized> Copy for ManualRef<T> {}

impl<T: ?Sized> Clone for ManualRef<T> {
    fn clone(&self) -> ManualRef<T> {
        *self
    }
}

impl<T: ?Sized + PartialEq> PartialEq for ManualRef<T> {
    fn eq(&self, other: &ManualRef<T>) -> bool {
        T::eq(self, other)
    }
}

impl<T: ?Sized + Eq> Eq for ManualRef<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for ManualRef<T> {
    fn partial_cmp(&self, other: &ManualRef<T>) -> Option<core::cmp::Ordering> {
        T::partial_cmp(self, other)
    }
}

impl<T: ?Sized + Ord> Ord for ManualRef<T> {
    fn cmp(&self, other: &ManualRef<T>) -> core::cmp::Ordering {
        T::cmp(self, other)
    }
}

impl<T: ?Sized + Hash> Hash for ManualRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self, state)
    }
}

impl<T: ?Sized + Debug> Debug for ManualRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized + Display> Display for ManualRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized> Deref for ManualRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: `ptr` is guaranteed to be valid for the lifetime of `self`, by
        // the constructor.
        unsafe { self.ptr.as_ref() }
    }
}

/// A mutable reference to a value of type `T`, without a lifetime.
pub struct ManualMut<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T: ?Sized> ManualMut<T> {
    /// Create a new `ManualMut` from a mutable reference.
    ///
    /// # Safety
    ///
    /// Erasing the lifetime of a reference is inherently safe. Because of this,
    /// this method is unsafe and the caller must ensure that Rust's reference
    /// lifetime rules are not violated.
    ///
    /// Among other things, this means that while the `ManualMut` exists, the data pointed
    /// to by `ptr` must not be moved or deallocated, and there may not be any other references
    /// to the data.
    pub const unsafe fn new(ptr: &mut T) -> ManualMut<T> {
        // SAFETY: `NonNull::new_unchecked` is safe to call with a pointer casted from a reference.
        unsafe {
            ManualMut {
                ptr: NonNull::new_unchecked(ptr as *mut _),
            }
        }
    }

    /// Create a new `ManualMut` from a static mutable reference.
    ///
    /// This is safe because the reference will never be invalidated.
    pub const fn from_static(ptr: &'static mut T) -> ManualMut<T> {
        // SAFETY: `ptr` has a static lifetime so it will always outlive the `ManualMut`.
        unsafe { Self::new(ptr) }
    }

    /// Create a new `ManualMut` from a raw mutable pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and aligned and point to a valid value of `T`.
    /// In addition, the same safety requirements as [`ManualMut::new`] applies.
    pub const unsafe fn from_ptr(ptr: *mut T) -> ManualMut<T> {
        // SAFETY: caller ensures that `ptr` is non-null.
        unsafe {
            ManualMut {
                ptr: NonNull::new_unchecked(ptr as *mut _),
            }
        }
    }

    /// Create a new `ManualMut` from a pinned mutable reference.
    ///
    /// # Safety
    ///
    /// This method has the same safety requirements as [`ManualRef::new`].
    pub const unsafe fn new_pinned(ptr: Pin<&mut T>) -> Pin<ManualMut<T>> {
        // SAFETY: `ManualMut<T>` preserves pinnability of `&mut T` and the caller ensures that
        // the safety requirements of `ManualMut::new` are met.
        unsafe { Pin::new_unchecked(ManualMut::new(Pin::into_inner_unchecked(ptr))) }
    }

    /// Convert to a raw pointer.
    pub const fn as_ptr(this: &Self) -> *mut T {
        this.ptr.as_ptr()
    }
}

// SAFETY: `ManualMut` has the same thread safety invariants as `&mut T`.
unsafe impl<T: ?Sized + Send> Send for ManualMut<T> {}

// SAFETY: `ManualMut` has the same thread safety invariants as `&mut T`.
unsafe impl<T: ?Sized + Sync> Sync for ManualMut<T> {}

impl<T: ?Sized + PartialEq> PartialEq for ManualMut<T> {
    fn eq(&self, other: &ManualMut<T>) -> bool {
        T::eq(self, other)
    }
}

impl<T: ?Sized + Eq> Eq for ManualMut<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for ManualMut<T> {
    fn partial_cmp(&self, other: &ManualMut<T>) -> Option<core::cmp::Ordering> {
        T::partial_cmp(self, other)
    }
}

impl<T: ?Sized + Ord> Ord for ManualMut<T> {
    fn cmp(&self, other: &ManualMut<T>) -> core::cmp::Ordering {
        T::cmp(self, other)
    }
}

impl<T: ?Sized + Hash> Hash for ManualMut<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self, state)
    }
}

impl<T: ?Sized + Debug> Debug for ManualMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized + Display> Display for ManualMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized> Deref for ManualMut<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: `ptr` is guaranteed to be valid for the lifetime of `self`, by
        // the constructor.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for ManualMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: `ptr` is guaranteed to be valid for the lifetime of `self`, by
        // the constructor.
        unsafe { self.ptr.as_mut() }
    }
}

/// A `'static` reference to a potentially non-`'static` type.
///
/// Use [`static_ref!`], [`StaticRef::from_static`], [`StaticRef::from_ptr`]
/// to create a `StaticRef`, and use [`StaticRef::as_ref`] to make a normal
/// reference.
pub struct StaticRef<T: ?Sized>(ManualRef<T>);

impl<T: ?Sized> StaticRef<T> {
    /// Create a new `StaticRef` from a normal static reference.
    pub const fn from_static(r: &'static T) -> StaticRef<T> {
        StaticRef(ManualRef::from_static(r))
    }

    /// Create a new `StaticRef` from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, aligned and point to a valid value of
    /// type `T`. It must be valid for the remainder of the program,
    /// just like a `&'static T`.
    pub const unsafe fn from_ptr(ptr: *const T) -> StaticRef<T> {
        // SAFETY: `r` points to a valid value of `T` and is valid for
        // the remained for the program.
        StaticRef(unsafe { ManualRef::from_ptr(ptr) })
    }

    /// Convert a `StaticRef` to a `&'a T` of any lifetime.
    pub const fn as_ref<'a>(this: Self) -> &'a T {
        // SAFETY: `self.0` points to a valid value of `T` and is valid
        // for the remained of the program.
        unsafe { &*ManualRef::as_ptr(&this.0) }
    }

    /// Cast a static ref to a different type.
    ///
    /// # Safety
    ///
    /// This function has the same safety requirements as
    /// ```ignore
    /// &*(self.to_ref() as *const T as *const U)
    /// ```
    pub const unsafe fn cast<U>(this: Self) -> StaticRef<U> {
        // SAFETY: caller ensures that `&T` can be cast to `&U`.
        StaticRef(unsafe { ManualRef::from_ptr(ManualRef::as_ptr(&this.0).cast::<U>()) })
    }
}

impl<T: ?Sized> Copy for StaticRef<T> {}

impl<T: ?Sized> Clone for StaticRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + PartialEq> PartialEq for StaticRef<T> {
    fn eq(&self, other: &StaticRef<T>) -> bool {
        T::eq(self, other)
    }
}

impl<T: ?Sized + Eq> Eq for StaticRef<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for StaticRef<T> {
    fn partial_cmp(&self, other: &StaticRef<T>) -> Option<core::cmp::Ordering> {
        T::partial_cmp(self, other)
    }
}

impl<T: ?Sized + Ord> Ord for StaticRef<T> {
    fn cmp(&self, other: &StaticRef<T>) -> core::cmp::Ordering {
        T::cmp(self, other)
    }
}

impl<T: ?Sized + Hash> Hash for StaticRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self, state)
    }
}

impl<T: ?Sized + Debug> Debug for StaticRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized + Display> Display for StaticRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized> Deref for StaticRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __static_ref {
    ($expr:expr) => {{
        let __ref = const { &$expr };

        // SAFETY: `__ref` is a const-evaluated reference, so it is necessarily static
        // even if it contains non-static values.
        unsafe { $crate::manual_ref::StaticRef::from_ptr(__ref) }
    }};
}

#[doc(inline)]
pub use __static_ref as static_ref;
