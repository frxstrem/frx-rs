//! Generic member access.

#![no_std]

mod request;

pub use self::request::Request;

/// Trait for types that provide generic member access.
pub trait Provide {
    /// Fulfill a request for generic member access.
    fn provide<'a>(&'a self, request: &mut Request<'a>);
}

/// Get a generic member from `src`.
///
/// See [`Provide`] and [`Request`] for details.
pub fn provide_ref<T: 'static>(src: &(impl Provide + ?Sized)) -> Option<&T> {
    let mut slot = request::RefSlot::new();
    let request = Request::new(&mut slot);

    src.provide(request);

    slot.take()
}

/// Get a generic member from `src`.
///
/// See [`Provide`] and [`Request`] for details.
pub fn provide_value<T: 'static>(src: &(impl Provide + ?Sized)) -> Option<T> {
    let mut slot = request::ValueSlot::new();
    let request = Request::new(&mut slot);

    src.provide(request);

    slot.take()
}
