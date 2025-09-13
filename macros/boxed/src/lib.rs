//! Auto-box function return types.
//!
//! See [`boxed`].

mod append_bounds;
mod args;
mod expand_async;
mod impl_to_dyn;
mod make_boxed;
mod utils;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use macro_helpers::{error::gather_errors_and_return, try_or_fail};
use syn::Error;

use self::{
    append_bounds::append_bounds, args::Args, expand_async::expand_async_fn,
    impl_to_dyn::impl_to_dyn_fn, make_boxed::make_boxed_fn,
};

/// Box the return type of the function.
///
/// # Pinning
///
/// When this macro is applied to a function using the `async` keyword, the
/// returned box will also be pinned.
///
/// For functions not marked `async`, pinning must be specified by using
/// `#[boxed(pin)]`.
///
/// ```ignore
/// #[boxed]
/// async fn foo() -> i32 { .. }
/// // will expand to:
/// //   fn foo() -> Pin<Box<impl Future<Output = i32>>> {
/// //     async move { .. }
/// //   }
///
/// #[boxed]
/// fn foo() -> impl Future<Output = i32> { .. }
/// // will expand to:
/// //   fn foo() -> Box<impl Future<Output = i32>> { .. }
///
/// #[boxed(pin)]
/// fn foo() -> impl Future<Output = i32> { .. }
/// // will expand to
/// //   fn foo() -> Pin<Box<impl Future<Output = i32>>> { .. }
/// ```
///
/// # Additional bounds
///
/// Using `#[boxed(bounds = Send + ..)]`, additional bounds can be added to
/// the return type.
///
/// ```ignore
/// #[boxed(bounds = Send)]
/// async fn foo() -> i32 { .. }
/// // will expand to:
/// //   fn foo() -> Pin<Box<impl Future<Output = i32> + Send>> {
/// //     async move { .. }
/// //   }
///
/// #[boxed(dyn, bounds = Send)]
/// async fn foo() -> i32 { .. }
/// // will expand to:
/// //   fn foo() -> Pin<Box<dyn Future<Output = i32> + Send>> {
/// //     async move { .. }
/// //   }
/// ```
///
/// # Trait objects
///
/// Using `#[boxed(dyn)]`, a returned `impl` trait will instead be turned into
/// a trait object.
///
/// ```ignore
/// #[boxed(dyn)]
/// async fn foo() -> i32 { .. }
/// // will expand to:
/// //   fn foo() -> Pin<Box<dyn Future<Output = i32>>> {
/// //     async move { .. }
/// //   }
///
///
/// #[boxed(pin, dyn, bound = Send)]
/// fn foo() -> impl Stream<Item = i32> { .. }
/// // will expand to:
/// //   fn foo() -> Pin<Box<dyn Stream<Item = i32> + Send>> {
/// //     async move { .. }
/// //   }
/// ```
#[proc_macro_attribute]
pub fn boxed(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args);
    let input = syn::parse_macro_input!(input);

    let (output, error) = gather_errors_and_return(|| boxed_impl(args, input));
    let error = error.map(Error::into_compile_error);
    quote! { #output #error }.into()
}

fn boxed_impl(args: TokenStream, mut input: syn::ItemFn) -> TokenStream {
    let mut args: Args = try_or_fail!(syn::parse2(args));

    expand_async_fn(&mut input, &mut args);
    append_bounds(&mut input, &args);
    impl_to_dyn_fn(&mut input, &args);
    make_boxed_fn(&mut input, &args);

    input.into_token_stream()
}
