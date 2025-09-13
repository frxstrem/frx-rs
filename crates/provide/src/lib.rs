//! Generic member access.

#![no_std]

mod macros;
mod request;

pub use self::request::Request;

/// Trait for types that provide generic member access.
pub trait Provide {
    /// Fulfill a request for generic member access.
    ///
    /// Implementors should call [`request.provide(value)`](Request::provide)
    /// or [`request.provide_with(value_fn)`](Request::provide_with).
    fn provide<'a>(&'a self, request: &mut Request<'a>);

    /// Fulfill a request for generic member access.
    ///
    /// Implementors should call [`request.provide(value)`](Request::provide)
    /// or [`request.provide_with(value_fn)`](Request::provide_with).
    fn provide_mut<'a>(&'a mut self, request: &mut Request<'a>) {
        self.provide(request)
    }
}

/// Get a generic member from `src`.
///
/// See [`Provide`] and [`Request`] for details.
pub fn provide<'a, T: 'a>(src: &'a (impl Provide + ?Sized)) -> Option<T> {
    let mut value = None::<T>;
    let mut request = Request::new(&mut value);

    src.provide(&mut request);

    value
}

/// Get a generic member from `src`.
///
/// See [`Provide`] and [`Request`] for details.
pub fn provide_mut<'a, T: 'a>(src: &'a mut (impl Provide + ?Sized)) -> Option<T> {
    let mut value = None::<T>;
    let mut request = Request::new(&mut value);

    src.provide_mut(&mut request);

    value
}
