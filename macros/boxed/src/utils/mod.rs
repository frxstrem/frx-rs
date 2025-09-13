pub mod lifetimes;

use quote::ToTokens;
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, Peek},
    parse_quote_spanned,
    spanned::Spanned,
};

pub fn make_return_type(sig: &mut syn::Signature) -> &mut syn::Type {
    let span = sig.span();

    if let syn::ReturnType::Default = &sig.output {
        sig.output = syn::ReturnType::Type(Token![->](span), parse_quote_spanned! {span=> () });
    }

    let syn::ReturnType::Type(_, ty) = &mut sig.output else {
        unreachable!()
    };
    ty
}

pub fn parse_list<P>(
    input: ParseStream,
    mut f: impl FnMut(ParseStream) -> Result<()>,
    _sep: P,
) -> Result<()>
where
    P: Peek,
    P::Token: Parse + ToTokens,
{
    while !input.is_empty() {
        f(input)?;

        if !input.is_empty() {
            let _ = input.parse::<P::Token>()?;
        }
    }

    Ok(())
}
