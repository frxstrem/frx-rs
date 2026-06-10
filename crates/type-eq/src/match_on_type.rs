//! Type matching.
//!
//! See [`match_on_type`] for usage.

use crate::type_eq::type_eq;

/// Construct a type match builder.
///
/// See [`MatchOnType`] for more details on usage.
pub fn match_on_type<T, R>(t: T) -> MatchOnType<T, R> {
    MatchOnType(Enum::Value(t))
}

/// A type match builder.
///
/// Use [`match_on_type`] to construct.
///
/// # Example
///
/// ```
/// # use type_eq::match_on_type;
/// fn f<T: 'static>(t: T) -> &'static str {
///     match_on_type(t)
///         .when::<i32>(|_n /* : i32 */| "i32")
///         .when::<&'static str>(|_s /* : &'static str */| "str")
///         .or("other")
/// }
///
/// assert_eq!(f(1i32), "i32");
/// assert_eq!(f("foo"), "str");
/// assert_eq!(f(false), "other");
/// ```
#[must_use]
pub struct MatchOnType<T, R>(Enum<T, R>);

enum Enum<T, R> {
    Value(T),
    Returns(R),
}

impl<T, R> MatchOnType<T, R> {
    fn map_inner(self, f: impl FnOnce(T) -> Result<R, T>) -> MatchOnType<T, R> {
        let Self(Enum::Value(t)) = self else {
            return self;
        };

        match f(t) {
            Ok(r) => Self(Enum::Returns(r)),
            Err(t) => Self(Enum::Value(t)),
        }
    }

    /// Define a branch for `T == U`.
    pub fn when<U>(self, f: impl FnOnce(U) -> R) -> MatchOnType<T, R>
    where
        T: 'static,
        U: 'static,
    {
        self.map_inner(|t| {
            if let Some(teq) = type_eq::<T, U>() {
                Ok(f(teq.transmute(t)))
            } else {
                Err(t)
            }
        })
    }

    /// Return the return value of the matched branch, or
    /// `default` if no branch was matched.
    pub fn or(self, default: R) -> R {
        match self.0 {
            Enum::Value(_) => default,
            Enum::Returns(r) => r,
        }
    }

    /// Return the return value of the matched branch, or
    /// the result of calling `default` if no branch was matched.
    pub fn or_else(self, default: impl FnOnce(T) -> R) -> R {
        match self.0 {
            Enum::Value(t) => default(t),
            Enum::Returns(r) => r,
        }
    }

    /// Return the return value of the matched branch, or
    /// `None` if no branch was matched.
    pub fn or_none(self) -> Option<R> {
        match self.0 {
            Enum::Value(_) => None,
            Enum::Returns(r) => Some(r),
        }
    }
}

impl<T> MatchOnType<T, ()> {
    /// Ignore any unhandled types.
    pub fn ignore_rest(self) {}
}

impl<T> MatchOnType<T, T> {
    /// Define a branch for `T == U`.
    ///
    /// The return type is automatically cast back into `T`.
    pub fn map_when<U>(self, f: impl FnOnce(U) -> U) -> MatchOnType<T, T>
    where
        T: 'static,
        U: 'static,
    {
        self.map_inner(|t| {
            if let Some(teq) = type_eq::<T, U>() {
                Ok(teq.transmute_back(f(teq.transmute(t))))
            } else {
                Err(t)
            }
        })
    }
}

impl<'a, T: ?Sized, R> MatchOnType<&'a T, R> {
    /// Define a branch for `&T == &U`.
    pub fn when_ref<U>(self, f: impl FnOnce(&'a U) -> R) -> MatchOnType<&'a T, R>
    where
        T: 'static,
        U: ?Sized + 'static,
    {
        self.map_inner(|t| {
            if let Some(teq) = type_eq::<T, U>() {
                Ok(f(teq.to_ref().transmute(t)))
            } else {
                Err(t)
            }
        })
    }
}

impl<'a, T: ?Sized, R> MatchOnType<&'a mut T, R> {
    /// Define a branch for `&mut T == &mut U`.
    pub fn when_mut<U>(self, f: impl FnOnce(&'a mut U) -> R) -> MatchOnType<&'a mut T, R>
    where
        T: 'static,
        U: ?Sized + 'static,
    {
        self.map_inner(|t| {
            if let Some(teq) = type_eq::<T, U>() {
                Ok(f(teq.to_mut().transmute(t)))
            } else {
                Err(t)
            }
        })
    }
}
