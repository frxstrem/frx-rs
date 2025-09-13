//! Type specialization on stable Rust.

#![no_std]

/// Branch an expression based on its type, as inferred at the macro callsite.
///
/// **Note:** While this macro can provide some level of type specialization,
/// it will only work in cases where the type can be inferred at the macro
/// call site. In particular, it can not be used to detect the type behind
/// an opaque type parameter.
///
/// # Usage
///
/// ```
/// # use core::fmt::Debug;
/// # use specialize::{macro_internals, specialize};
/// # trait MyTrait {}
/// # struct MyDefaultImpl;
/// # impl MyTrait for MyDefaultImpl {}
/// # type T = i32;
/// # let input = 1;
///
/// specialize!(
///     // `input` is the name of a variable being matched on.
///     match input {
///         // Branches have the form `<type>  =>  <expr>`.
///         &'static str => "str".to_string(),
///
///         // Inside of `<expr>` the variable will have the type of the branch arm.
///         i32 => format!("{}", input * input),
///
///         // `impl Trait` is supported.
///         impl Debug => format!("debug: {input:?}"),
///
///         // ...as well as generic types.
///         for<T> Option<T> => "option".to_string(),
///
///         // If `Trait` is not implemented for `!`, then it may be necessary to
///         // specify a different type for inferrence using `default <type>`.
///         // This type is arbitrary as long as it implements the trait.
///         impl MyTrait default MyDefaultImpl => "my_trait".to_string(),
///
///         // Lastly a fallback branch is always required.
///         _ => "(fallback)".to_string()
///     }
/// );
//
/// specialize!(
///     // It is also possible to match on a named type by using the
///     // `match type <type>` syntax. It is not possible to directly access
///     // the more specific type inside of the branches, though.
///     match type T {
///         impl Send => true,
///         _ => false,
///     }
/// );
/// ```
pub use specialize_macro::specialize;

#[doc(hidden)]
pub mod macro_internals {
    pub trait Specialization<T>: MatchResult {}

    pub trait MatchResult: Sized {
        const SINGLETON: Option<Self>;
    }

    pub const fn match_specialization<S, F, T, M: MatchResult>(
        _: fn(Infer<S, F, T>) -> M,
    ) -> Option<M> {
        M::SINGLETON
    }

    pub const fn match_specialization_of<S, F, T, M: MatchResult>(
        _: &T,
        _: fn(Infer<S, F, T>) -> M,
    ) -> Option<M> {
        M::SINGLETON
    }

    pub struct Infer<S, F, T>(S, F, T);

    impl<S: Specialization<T>, F, T> Infer<S, F, T> {
        pub fn infer_match(&self) -> S {
            unreachable!()
        }
    }

    pub trait InferFallback<F> {
        fn infer_match(&self) -> F {
            unreachable!()
        }
    }

    impl<S, F, T> InferFallback<F> for Infer<S, F, T> {}
}
