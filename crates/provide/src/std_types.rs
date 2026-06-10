//! `Provide` implementations for standard library types.

use crate::{Provide, Request};

impl Provide for core::convert::Infallible {
    fn provide<'a>(&'a self, _request: &mut Request<'a>) {
        match *self {}
    }
}

impl<P: ?Sized + Provide> Provide for &P {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

impl<P: ?Sized + Provide> Provide for &mut P {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

#[cfg(feature = "alloc")]
impl<P: ?Sized + Provide> Provide for alloc::boxed::Box<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

#[cfg(feature = "alloc")]
impl<P: ?Sized + Provide> Provide for alloc::rc::Rc<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

#[cfg(feature = "alloc")]
impl<P: ?Sized + Provide> Provide for alloc::sync::Arc<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

impl<P: Provide> Provide for Option<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        if let Some(inner) = self {
            P::provide(inner, request)
        }
    }
}

impl<P: Provide, E: Provide> Provide for Result<P, E> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        match self {
            Ok(inner) => P::provide(inner, request),
            Err(err) => E::provide(err, request),
        }
    }
}

impl<P: core::ops::Deref<Target: Provide>> Provide for core::pin::Pin<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::Target::provide(self, request)
    }
}

impl<P: ?Sized + Provide> Provide for core::cell::Ref<'_, P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

impl<P: ?Sized + Provide> Provide for core::cell::RefMut<'_, P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

#[cfg(feature = "std")]
impl<P: ?Sized + Provide> Provide for std::sync::MutexGuard<'_, P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

#[cfg(feature = "std")]
impl<P: ?Sized + Provide> Provide for std::sync::RwLockReadGuard<'_, P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

#[cfg(feature = "std")]
impl<P: ?Sized + Provide> Provide for std::sync::RwLockWriteGuard<'_, P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

impl<P: Provide> Provide for core::cell::OnceCell<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        if let Some(inner) = self.get() {
            P::provide(inner, request)
        }
    }
}

impl<P: Provide> Provide for core::cell::LazyCell<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}

#[cfg(feature = "std")]
impl<P: Provide> Provide for std::sync::OnceLock<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        if let Some(inner) = self.get() {
            P::provide(inner, request)
        }
    }
}

#[cfg(feature = "std")]
impl<P: Provide> Provide for std::sync::LazyLock<P> {
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        P::provide(self, request)
    }
}
