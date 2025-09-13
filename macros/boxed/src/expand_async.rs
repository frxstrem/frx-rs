use proc_macro2::Span;
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned};

use crate::utils::make_return_type;

use super::args::{self, Args};

/// Desugar `async fn(..) -> R` into `fn(..) -> impl Future<Output = R>`.
pub fn expand_async_fn(func: &mut syn::ItemFn, args: &mut Args) {
    if func.sig.asyncness.is_none() {
        return;
    }

    func.attrs.push(parse_quote! {
        #[allow(clippy::manual_async_fn)]
    });

    func.sig.asyncness = None;
    args.pin = Some(args::kw::pin(Span::call_site()));

    let ty = make_return_type(&mut func.sig);

    *ty = syn::Type::ImplTrait(parse_quote_spanned! {ty.span()=>
        impl ::core::future::Future<Output = #ty>
    });

    let block = &func.block;
    func.block = parse_quote_spanned! {block.span()=>
        { async move #block }
    };
}
