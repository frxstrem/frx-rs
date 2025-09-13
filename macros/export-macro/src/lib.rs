//! Auto-export `macro_rules!` macros.
//!
//! See [`export_macro`]

use macro_helpers::{error::gather_errors_and_return, fail};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, ItemMacro, Result, Visibility,
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned,
    spanned::Spanned,
};

/// Auto-export `macro_rules!` macros.
///
/// The default is to export as `pub(self)`, but this can be
/// specified by passing the visibility as an argument, such
/// as `#[export_macro(pub)]` or `#[export_macro(pub(super))]`.
#[proc_macro_attribute]
pub fn export_macro(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args);
    let input = syn::parse_macro_input!(input);

    let (output, error) = gather_errors_and_return(|| export_macro_impl(args, input));
    let error = error.map(Error::into_compile_error);
    quote! { #output #error }.into()
}

fn export_macro_impl(args: Args, mut input: ItemMacro) -> TokenStream {
    let vis = args.vis;

    if !input.mac.path.is_ident("macro_rules") {
        fail!(
            input.mac.path.span(),
            "#[export_macro]: can only be applied to macro_rules!",
            return input,
        );
    }

    let Some(macro_name) = &input.ident else {
        fail!(
            input.span(),
            "#[export_macro]: missing macro name",
            return input,
        );
    };

    if matches!(vis, Visibility::Public(_)) {
        input.attrs.push(parse_quote_spanned! {vis.span()=>
            #[macro_export]
        });
    }

    let inner_vis: Visibility = match vis {
        Visibility::Public(pub_) => Visibility::Public(pub_),
        Visibility::Inherited | Visibility::Restricted(_) => {
            parse_quote! { pub(crate) }
        }
    };

    let mod_name = format_ident!("__export_macro__{macro_name}");

    quote! {
        mod #mod_name {
            use super::*;

            #input

            #inner_vis use #macro_name;
        }

        #vis use #mod_name::*;
    }
}

struct Args {
    vis: Visibility,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            vis: input.parse()?,
        })
    }
}
