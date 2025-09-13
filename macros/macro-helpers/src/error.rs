//! Proc macro error handling.

use std::{
    cell::RefCell,
    fmt::Display,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

use proc_macro2::Span;
use quote::ToTokens;
use syn::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

/// Builder for parser errors.
#[derive(Default)]
pub struct ErrorBuilder {
    errors: Vec<Error>,
}

impl ErrorBuilder {
    /// Create a new `ErrorBuilder`.
    pub const fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add a new error to the `ErrorBuilder`.
    pub fn push(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Add a new error to the `ErrorBuilder`.
    ///
    /// This is a wrapper for `self.push(syn::Error::new(..))`.
    pub fn push_new<T: Display>(&mut self, span: Span, message: T) {
        self.push(Error::new(span, message))
    }

    /// Add a new spanned error to the `ErrorBuilder`.
    ///
    /// This is a wrapper for `self.push(syn::Error::new_spanned(..))`.
    pub fn push_new_spanned<T: ToTokens, U: Display>(&mut self, tokens: T, message: U) {
        self.push(Error::new_spanned(tokens, message))
    }

    /// Convert the `ErrorBuilder` into a `Result<(), syn::Error>`.
    pub fn into_result(self) -> Result<()> {
        self.into_option().map_or(Ok(()), Err)
    }

    /// Convert the `ErrorBuilder` into an `Option<syn::Error>`.
    pub fn into_option(self) -> Option<Error> {
        self.errors.into_iter().reduce(|mut rhs, lhs| {
            rhs.combine(lhs);
            rhs
        })
    }
}

thread_local! {
    static ERROR: RefCell<ErrorBuilder> = const { RefCell::new(ErrorBuilder::new()) };
}

/// Emit an error to be collected later.
pub fn emit_error(err: Error) {
    ERROR.with_borrow_mut(|b| b.push(err))
}

/// Run a closure, gathering any errors emitted with [`emit_error`].
pub fn gather_errors_and_return<F: FnOnce() -> R, R>(f: F) -> (R, Option<Error>) {
    let old_error = ERROR.replace(ErrorBuilder::new());
    let result = catch_unwind(AssertUnwindSafe(f));
    let new_error = ERROR.replace(old_error);

    match result {
        Ok(output) => (output, new_error.into_option()),
        Err(payload) => resume_unwind(payload),
    }
}

/// Run a closure, gathering any errors emitted with [`emit_error`].
pub fn gather_errors<F: FnOnce()>(f: F) -> Result<(), Error> {
    let old_error = ERROR.replace(ErrorBuilder::new());
    let result = catch_unwind(AssertUnwindSafe(f));
    let new_error = ERROR.replace(old_error);

    match result {
        Ok(()) => new_error.into_result(),
        Err(payload) => resume_unwind(payload),
    }
}

/// Extension trait to emit an error or result.
///
/// This is implemented for [`syn::Error`], [`ErrorBuilder`], `Option<syn::Error>`
/// and `Result<T, syn::Error>`.
pub trait EmitError {
    /// The result of [`self.emit_error()`](EmitError::emit_error).
    type Out;

    /// If `self` contains an error, emit it.
    fn emit_error(self) -> Self::Out;
}

impl EmitError for Error {
    type Out = ();

    fn emit_error(self) {
        emit_error(self);
    }
}

impl<T> EmitError for Result<T, Error> {
    type Out = Option<T>;

    fn emit_error(self) -> Option<T> {
        self.map_err(emit_error).ok()
    }
}

impl EmitError for Option<Error> {
    type Out = ();

    fn emit_error(self) {
        if let Some(inner) = self {
            emit_error(inner)
        }
    }
}

impl EmitError for ErrorBuilder {
    type Out = ();

    fn emit_error(self) -> Self::Out {
        self.into_option().emit_error()
    }
}

/// Emit an error and return.
///
/// Two different syntaxes are supported:
/// ```ignore
/// fail!(<err>, return <output>);
/// fail!(<span>, <message>, return <output>);
/// ```
///
/// `<err>` is the emitted error and `<output>` is the output that will
/// be returned as a token stream. `return <output>` can be omitted, in which
/// case an empty token stream is returned.
///
/// `<span>` and `<message>` can be used instead of `<err>`, which calls
/// [`syn::Error::new`] to create a new error.
///
/// See [`emit_error`].
#[macro_export]
macro_rules! fail {
    ($err:expr $(,)?) => {
        return {
            let _ = $crate::error::EmitError::emit_error($err);
            proc_macro2::TokenStream::new()
        }
    };
    ($err:expr, return $returns:expr $(,)?) => {
        return {
            let _ = $crate::error::EmitError::emit_error($err);
            quote::ToTokens::into_token_stream($returns)
        }
    };
    ($span:expr, $message:expr $(, $($rest:tt)*)?) => {
        $crate::fail! { syn::Error::new($span, $message) $(,$($rest)*)? }
    };
}

/// If an expression fails, emit the error and return.
///
/// Syntax:
/// ```ignore
/// try_or_fail!(<expr>, return <output>);
/// ```
///
/// `try_or_fail!` acts similar to the `?` macro, except that if `<expr>`
/// is an `Err(_)`, the error is emitted and `<output>` is returned as a
/// token stream. `return <output>` can be omitted, in which case
/// an empty token stream is returned.
///
/// See [`emit_error`].
#[macro_export]
macro_rules! try_or_fail {
    ($result:expr $(,)?) => {
        match $result {
            Ok(__output) => __output,
            Err(__err) => {
                let _ = $crate::error::EmitError::emit_error(__err);
                return proc_macro2::TokenStream::new();
            }
        }
    };
    ($result:expr, return $returns:expr $(,)?) => {
        match $result {
            Ok(__output) => __output,
            Err(__err) => {
                let _ = $crate::error::EmitError::emit_error(__err);
                return quote::ToTokens::into_token_stream($returns);
            }
        }
    };
}
