#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{marker::PhantomData, mem::MaybeUninit, ops::Deref, ptr::NonNull};

/// Compare two types, returning a [`TypeEq`] if they are equal.
#[cfg(not(feature = "nightly"))]
pub fn type_eq<T: 'static, U: 'static>() -> Option<TypeEq<T, U>> {
    use core::any::TypeId;

    if TypeId::of::<T>() == TypeId::of::<U>() {
        // SAFETY: `T` and `U` are the same type.
        Some(unsafe { TypeEq::new_unchecked() })
    } else {
        None
    }
}

/// Compare two types, returning a [`TypeEq`] if they are equal.
#[cfg(feature = "nightly")]
pub const fn type_eq<T: 'static, U: 'static>() -> Option<TypeEq<T, U>> {
    const fn is_type_eq<T: 'static, U: 'static>() -> bool {
        trait Is<U: ?Sized> {
            const IS_SAME: bool;
        }

        impl<T: ?Sized, U: ?Sized> Is<U> for T {
            default const IS_SAME: bool = false;
        }

        impl<T: ?Sized> Is<T> for T {
            const IS_SAME: bool = true;
        }

        <T as Is<U>>::IS_SAME
    }

    if is_type_eq::<T, U>() {
        // SAFETY: `T` and `U` are the same type.
        Some(unsafe { TypeEq::new_unchecked() })
    } else {
        None
    }
}

/// A typed assertion that `T` and `U` represent the same types.
pub struct TypeEq<T: ?Sized, U: ?Sized> {
    _t: PhantomData<fn(T) -> T>,
    _u: PhantomData<fn(U) -> U>,
}

impl<T: ?Sized, U: ?Sized> TypeEq<T, U> {
    /// Construct a new `TypeEq<T, U>`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `T` and `U` are the exact same type.
    pub const unsafe fn new_unchecked() -> TypeEq<T, U> {
        TypeEq {
            _t: PhantomData,
            _u: PhantomData,
        }
    }

    /// Flip the order of the type equality.
    pub const fn flip(self) -> TypeEq<U, T> {
        // SAFETY: If `T == U` then `U == T`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert the type equality to one between references.
    pub const fn to_ref<'a>(self) -> TypeEq<&'a T, &'a U> {
        // SAFETY: If `T == U` then `&T == &U`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert the type equality to one between mutable references.
    pub const fn to_mut<'a>(self) -> TypeEq<&'a mut T, &'a mut U> {
        // SAFETY: If `T == U` then `&mut T == &mut U`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert the type equality to one between raw pointers.
    pub const fn to_ptr(self) -> TypeEq<*const T, *const U> {
        // SAFETY: If `T == U` then `*const T == *const U`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert the type equality to one between mutable raw pointers.
    pub const fn to_mut_ptr(self) -> TypeEq<*mut T, *mut U> {
        // SAFETY: If `T == U` then `*mut T == *mut U`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert the type equality to one between `NonNull`s.
    pub const fn to_non_null(self) -> TypeEq<NonNull<T>, NonNull<U>> {
        // SAFETY: If `T == U` then `NonNull<T> == NonNull<U>`.
        unsafe { TypeEq::new_unchecked() }
    }

    #[cfg(feature = "alloc")]
    /// Convert the type equality to one between boxes.
    pub const fn to_box(self) -> TypeEq<Box<T>, Box<U>> {
        // SAFETY: If `T == U` then `Box<T> == Box<U>`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<T: ?Sized + Deref, U: ?Sized + Deref> TypeEq<T, U> {
    /// Convert the type equality to one between the dereferenced targets.
    pub const fn to_deref(self) -> TypeEq<T::Target, U::Target> {
        // SAFETY: If `T == U` then `T::Target == U::Target`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<T: ?Sized, U: ?Sized> TypeEq<*const T, *const U> {
    /// Convert the type equality to one between the dereferenced targets.
    pub const fn to_raw_deref(self) -> TypeEq<T, U> {
        // SAFETY: If `*const T == *const U` then `T == U`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<T: ?Sized, U: ?Sized> TypeEq<*mut T, *mut U> {
    /// Convert the type equality to one between the dereferenced targets.
    pub const fn to_raw_deref(self) -> TypeEq<T, U> {
        // SAFETY: If `*mut T == *mut U` then `T == U`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<T: ?Sized, U: ?Sized> TypeEq<NonNull<T>, NonNull<U>> {
    /// Convert the type equality to one between the dereferenced targets.
    pub const fn to_raw_deref(self) -> TypeEq<T, U> {
        // SAFETY: If `NonNull<T> == NonNull<U>` then `T == U`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<T, U> TypeEq<T, U> {
    /// Transmute a value of type `T` to type `U`.
    ///
    /// This is sound because a reference to `TypeEq<T, U>` can only exist
    /// if `T` and `U` are the same type.
    pub const fn transmute(&self, t: T) -> U {
        let t = MaybeUninit::new(t);
        // SAFETY: If `T == U` then transmuting `T` to `U` is sound.
        unsafe { t.as_ptr().cast::<U>().read() }
    }

    /// Transmute a value of type `U` back to type `T`.
    ///
    /// This is sound because a reference to `TypeEq<T, U>` can only exist
    /// if `T` and `U` are the same type.
    pub const fn transmute_back(&self, u: U) -> T {
        self.flip().transmute(u)
    }
}

impl<T: ?Sized, U: ?Sized> Copy for TypeEq<T, U> {}

impl<T: ?Sized, U: ?Sized> Clone for TypeEq<T, U> {
    fn clone(&self) -> Self {
        *self
    }
}
