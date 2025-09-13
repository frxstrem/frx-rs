use core::{any::TypeId, marker::PhantomData, ptr::NonNull};

use crate::provide_match;

/// A request to provide generic member access.
///
/// See [`Provide`] for details.
pub struct Request<'a> {
    _lifetime: PhantomData<&'a ()>,
    type_id: TypeId,
    ptr: NonNull<()>,
}

impl<'a> Request<'a> {
    pub(crate) fn new<T: 'a>(slot: &'a mut Option<T>) -> Self {
        Self {
            _lifetime: PhantomData,
            type_id: typeid::of::<T>(),
            ptr: NonNull::from(slot).cast::<()>(),
        }
    }

    fn get<T: 'a>(&self) -> Option<&Option<T>> {
        if self.type_id == typeid::of::<T>() {
            // SAFETY: we have checked that the pointer points to the right type
            Some(unsafe { self.ptr.cast::<Option<T>>().as_ref() })
        } else {
            None
        }
    }

    fn get_mut<T: 'a>(&mut self) -> Option<&mut Option<T>> {
        if self.type_id == typeid::of::<T>() {
            // SAFETY: we have checked that the pointer points to the right type
            Some(unsafe { self.ptr.cast::<Option<T>>().as_mut() })
        } else {
            None
        }
    }

    /// Checks whether the request would be satisified by the given type.
    ///
    /// Returns true if `T` is the type expected to be provided and
    /// the request has not already been satisfied. If this method
    /// returns true, [`self.provide()`](Self::provide) and
    /// [`self.provide_with()`](Self::provide_with) will do nothing.
    pub fn would_be_satisfied_by<T: 'a>(&self) -> bool {
        self.get::<T>().is_some_and(|slot| slot.is_none())
    }

    /// Provide the given value if it would satisfy the request.
    pub fn provide<T: 'a>(&mut self, value: T) -> &mut Self {
        self.provide_with(|| value)
    }

    /// Provide the value produced by the given closure if it would satisfy
    /// the request.
    pub fn provide_with<T: 'a>(&mut self, fulfil: impl FnOnce() -> T) -> &mut Self {
        if let Some(slot) = self.get_mut::<T>() {
            slot.get_or_insert_with(fulfil);
        }

        self
    }

    /// Provide the given reference as a `&mut` or `&` reference.
    ///
    /// `request.provide_mut::<T>(&mut x)` is effectively the same as
    /// ```ignore
    /// request.provide::<&mut T>(&mut x);
    /// request.provide::<&T>(&x);
    /// ```
    /// except it does not run into lifetime issues from borrowing twice.
    pub fn provide_mut<T: ?Sized + 'a>(&mut self, value: &'a mut T) -> &mut Self {
        provide_match!(self,
            &mut T => value,
            &T => value,
        )
    }
}
