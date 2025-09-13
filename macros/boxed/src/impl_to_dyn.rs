use std::mem;

use macro_helpers::error::EmitError;
use proc_macro2::{Span, TokenStream};
use syn::{Error, Token, parse_quote_spanned, spanned::Spanned};

use crate::utils::{
    lifetimes::{expand_elided_lifetimes, find_lifetimes},
    make_return_type,
};

use super::Args;

/// Replace an `impl Trait` return type with a `dyn Trait` return type.
pub fn impl_to_dyn_fn(func: &mut syn::ItemFn, args: &Args) {
    let Some(dyn_) = &args.dyn_ else {
        return;
    };

    let ty = make_return_type(&mut func.sig);

    let syn::Type::ImplTrait(impl_trait) = ty else {
        Error::new_spanned(
            dyn_,
            "`#[boxed(dyn)` can only be used on functions that return `impl Trait`",
        )
        .emit_error();
        return;
    };

    // if there are any precise capture bounds, fail.
    let precise_capture_bound = impl_trait
        .bounds
        .iter()
        .find(|bound| matches!(bound, syn::TypeParamBound::PreciseCapture(_)));

    if let Some(bound) = precise_capture_bound {
        Error::new_spanned(
            bound,
            "cannot convert `impl` to `dyn` with precise capture bound",
        )
        .emit_error();
        return;
    }

    // if there are any lifetime bounds, we can simply convert
    // from `impl` to `dyn`.
    let has_lifetime_bound = impl_trait
        .bounds
        .iter()
        .any(|bound| matches!(bound, syn::TypeParamBound::Lifetime(_)));

    if has_lifetime_bound {
        replace_impl_with_dyn(ty);
        return;
    }

    // otherwise, expand all lifetimes, then create a new output lifetime
    let output_lifetime = syn::Lifetime::new("'__output", Span::call_site());
    impl_trait.bounds.insert(0, output_lifetime.clone().into());

    expand_elided_lifetimes(func).emit_error();

    let arg_lifetimes = find_lifetimes(|v| {
        for input in &func.sig.inputs {
            v.visit_fn_arg(input);
        }
    });

    let reuse_output_lifetime = match arg_lifetimes.len() {
        0 => Some(syn::Lifetime::new("'static", output_lifetime.span())),

        1 => Some(arg_lifetimes[0].clone()),

        2.. => None,
    };

    if let Some(reuse_output_lifetime) = reuse_output_lifetime {
        let syn::Type::ImplTrait(impl_trait) = make_return_type(&mut func.sig) else {
            panic!("internal error");
        };

        impl_trait.bounds.iter_mut().for_each(|bound| {
            if let syn::TypeParamBound::Lifetime(lt) = bound
                && lt.ident == output_lifetime.ident
            {
                *lt = reuse_output_lifetime.clone();
            }
        });
    } else {
        func.sig.generics.params.insert(
            0,
            syn::GenericParam::Lifetime(syn::LifetimeParam::new(output_lifetime.clone())),
        );

        func.sig
            .generics
            .make_where_clause()
            .predicates
            .extend(arg_lifetimes.into_iter().map(|lt| -> syn::WherePredicate {
                parse_quote_spanned! {output_lifetime.span()=> #lt: #output_lifetime }
            }));
    }

    replace_impl_with_dyn(make_return_type(&mut func.sig));
}

fn replace_impl_with_dyn(ty: &mut syn::Type) {
    if let syn::Type::ImplTrait(..) = ty {
        let syn::Type::ImplTrait(syn::TypeImplTrait { impl_token, bounds }) =
            mem::replace(ty, syn::Type::Verbatim(TokenStream::new()))
        else {
            unreachable!()
        };

        *ty = syn::Type::TraitObject(syn::TypeTraitObject {
            dyn_token: Some(Token![dyn](impl_token.span())),
            bounds,
        })
    }
}
