use core::{any::TypeId, ptr};

/// A request to provide generic member access.
///
/// See [`Provide`] for details.
#[repr(transparent)]
pub struct Request<'a> {
    slot: dyn 'a + Slot<'a>,
}

impl<'a> Request<'a> {
    pub(crate) fn new<S: Slot<'a> + 'a>(slot: &mut S) -> &mut Self {
        let slot: &mut (dyn 'a + Slot) = slot;

        // SAFETY: `Request` is `repr(transparent)` and `slot` is a valid
        // reference to a `dyn Slot`.
        unsafe { &mut *(ptr::from_mut(slot) as *mut Self) }
    }

    /// Check if the request has already been satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.slot.is_satisfied()
    }

    /// Check if the request would be satisfied by calling
    /// `provide_ref::<T>`.
    pub fn would_be_satisfied_by_ref<T: 'static>(&self) -> bool {
        self.slot
            .as_ref_slot::<T>()
            .is_some_and(|slot| !slot.is_satisfied())
    }

    /// Check if the request would be satisfied by calling
    /// `provide_value::<T>`.
    pub fn would_be_satisfied_by_value<T: 'static>(&self) -> bool {
        self.slot
            .as_value_slot::<T>()
            .is_some_and(|slot| !slot.is_satisfied())
    }

    /// Provide a reference to satisfy the request.
    pub fn provide_ref<T: 'static>(&mut self, ref_: &'a T) {
        self.provide_ref_with(|| ref_)
    }

    /// Provide a reference to satisfy the request.
    ///
    /// Unlike `provide_ref`, this function will only lazily evaluate the
    /// provided closure if the request would be satisfied by a reference to `T`.
    pub fn provide_ref_with<T: 'static>(&mut self, ref_fn: impl FnOnce() -> &'a T) {
        if let Some(RefSlot(slot)) = self.slot.as_ref_slot_mut() {
            slot.get_or_insert_with(ref_fn);
        };
    }

    /// Provide a reference to satisfy the request.
    ///
    /// Unlike `provide_ref_with`, this function will only evaluate the
    /// provided closure if the request would be satisfied by a reference to `T`.
    ///
    /// # Errors
    ///
    /// If the request would be satisfied by a value of `T`, but the closure
    /// returns an error, then the error will be returned and the request
    /// will remain unsatisfied..
    pub fn try_provide_ref_with<T: 'static, E>(
        &mut self,
        ref_fn: impl FnOnce() -> Result<&'a T, E>,
    ) -> Result<(), E> {
        if let Some(RefSlot(slot)) = self.slot.as_ref_slot_mut()
            && slot.is_none()
        {
            *slot = Some(ref_fn()?);
        };
        Ok(())
    }

    /// Provide a value to satisfy the request.
    pub fn provide_value<T: 'static>(&mut self, value: T) {
        self.provide_value_with(|| value)
    }

    /// Provide a value to satisfy the request.
    ///
    /// Unlike `provide_value`, this function will only lazily evaluate the
    /// provided closure if the request would be satisfied by a value of `T`.
    pub fn provide_value_with<T: 'static>(&mut self, value_fn: impl FnOnce() -> T) {
        if let Some(ValueSlot(slot)) = self.slot.as_value_slot_mut::<T>() {
            slot.get_or_insert_with(value_fn);
        };
    }

    /// Provide a value to satisfy the request.
    ///
    /// Unlike `provide_value_with`, this function will only evaluate the
    /// provided closure if the request would be satisfied by a value of `T`.
    ///
    /// # Errors
    ///
    /// If the request would be satisfied by a value of `T`, but the closure
    /// returns an error, then the error will be returned and the request
    /// will remain unsatisfied.
    pub fn try_provide_value_with<T: 'static, E>(
        &mut self,
        value_fn: impl FnOnce() -> Result<T, E>,
    ) -> Result<(), E> {
        if let Some(ValueSlot(slot)) = self.slot.as_value_slot_mut::<T>()
            && slot.is_none()
        {
            *slot = Some(value_fn()?);
        };
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum SlotType {
    /// `Option<&'a T>`
    Ref,
    /// `Option<T>`
    Value,
}

/// A type-erased `Option<T>` or `Option<&'a T>`, where `T: 'static`.
///
/// # Safety
///
/// The values returned by `slot_type()` and `value_type()` must match `Self`.
pub(crate) unsafe trait Slot<'a> {
    fn slot_type(&self) -> SlotType;
    fn value_type(&self) -> TypeId;
    fn is_satisfied(&self) -> bool;
}

impl<'a> dyn 'a + Slot<'a> {
    fn as_ref_slot<T: 'static>(&self) -> Option<&RefSlot<'a, T>> {
        (self.slot_type() == SlotType::Ref && self.value_type() == TypeId::of::<T>())
            .then(|| unsafe { &*(ptr::from_ref(self) as *const RefSlot<'a, T>) })
    }

    fn as_ref_slot_mut<T: 'static>(&mut self) -> Option<&mut RefSlot<'a, T>> {
        (self.slot_type() == SlotType::Ref && self.value_type() == TypeId::of::<T>())
            .then(|| unsafe { &mut *(ptr::from_mut(self) as *mut RefSlot<'a, T>) })
    }

    fn as_value_slot<T: 'static>(&self) -> Option<&ValueSlot<T>> {
        (self.slot_type() == SlotType::Value && self.value_type() == TypeId::of::<T>())
            .then(|| unsafe { &*(ptr::from_ref(self) as *const ValueSlot<T>) })
    }

    fn as_value_slot_mut<T: 'static>(&mut self) -> Option<&mut ValueSlot<T>> {
        (self.slot_type() == SlotType::Value && self.value_type() == TypeId::of::<T>())
            .then(|| unsafe { &mut *(ptr::from_mut(self) as *mut ValueSlot<T>) })
    }
}

pub(crate) struct RefSlot<'a, T: 'static>(Option<&'a T>);

impl<'a, T: 'static> RefSlot<'a, T> {
    pub(crate) fn new() -> Self {
        Self(None)
    }

    pub(crate) fn take(self) -> Option<&'a T> {
        self.0
    }
}

unsafe impl<'a, T: 'static> Slot<'a> for RefSlot<'a, T> {
    fn slot_type(&self) -> SlotType {
        SlotType::Ref
    }

    fn value_type(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn is_satisfied(&self) -> bool {
        self.0.is_some()
    }
}

pub(crate) struct ValueSlot<T: 'static>(Option<T>);

impl<T: 'static> ValueSlot<T> {
    pub(crate) fn new() -> Self {
        Self(None)
    }

    pub(crate) fn take(self) -> Option<T> {
        self.0
    }
}

unsafe impl<T: 'static> Slot<'_> for ValueSlot<T> {
    fn slot_type(&self) -> SlotType {
        SlotType::Value
    }

    fn value_type(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn is_satisfied(&self) -> bool {
        self.0.is_some()
    }
}
