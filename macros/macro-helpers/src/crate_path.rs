//! Resolve path to crate items.
//!
//! See [`crate_path`].

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;

/// Resolve a path to a crate item.
pub fn crate_path(path: &str, span: Span) -> TokenStream {
    let segments = path
        .strip_prefix("::")
        .unwrap_or(path)
        .split("::")
        .map(str::trim)
        .collect::<Vec<_>>();

    let (root, segments) = segments.split_first().expect("empty crate path");

    let root = match crate_name(root) {
        Ok(FoundCrate::Itself) => syn::Ident::new("crate", span),
        Ok(FoundCrate::Name(root)) => syn::Ident::new(&root, span),
        Err(_) => syn::Ident::new(root, span),
    };

    let segments = segments.iter().map(|s| syn::Ident::new(s, span));

    quote! {
        #root #(:: #segments)*
    }
}
