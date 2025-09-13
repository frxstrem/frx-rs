use proc_macro2::Span;
use syn::{ExprBlock, Token, parse_quote_spanned, spanned::Spanned};

use crate::utils::make_return_type;

use super::Args;

/// Wrap the return type in a `Box`.
///
/// If `args.pinned` is set, the return type will be wrapped in a pinned `Box`.
pub fn make_boxed_fn(func: &mut syn::ItemFn, args: &Args) {
    let ty = make_return_type(&mut func.sig);
    *ty = box_type(ty, args);

    *func.block = box_block((*func.block).clone(), args);
}

fn get_box_type(_args: &Args, span: Span) -> syn::Path {
    parse_quote_spanned! {span=> ::std::boxed::Box }
}

fn get_box_constructor(args: &Args, span: Span) -> syn::Path {
    let mut box_type = get_box_type(args, span);
    let span = box_type.span();

    let box_method_name = if let Some(pin) = args.pin {
        syn::Ident::new("pin", pin.span())
    } else {
        syn::Ident::new("new", span)
    };

    box_type.segments.push_punct(Token![::](span));
    box_type.segments.push(box_method_name.into());

    box_type
}

fn box_block(block: syn::Block, args: &Args) -> syn::Block {
    let span = block.span();

    let box_constructor = get_box_constructor(args, span);

    let expr = wrap_block(block);

    parse_quote_spanned! {span=>
        {
            #box_constructor(#expr)
        }
    }
}

fn box_type(ty: &syn::Type, args: &Args) -> syn::Type {
    let pin_type: Option<syn::Path> = args.pin.as_ref().map(|pin| {
        parse_quote_spanned! {pin.span()=>
            ::core::pin::Pin
        }
    });

    let box_type = get_box_type(args, ty.span());

    if let Some(pin_type) = pin_type {
        parse_quote_spanned! {ty.span()=>
            #pin_type::<#box_type::<#ty>>
        }
    } else {
        parse_quote_spanned! {ty.span()=>
            #box_type::<#ty>
        }
    }
}

fn wrap_block(block: syn::Block) -> syn::Expr {
    let expr = if block.stmts.len() == 1
        && let Some(syn::Stmt::Expr(_expr, None)) = block.stmts.first()
    {
        let Some(syn::Stmt::Expr(expr, _)) = block.stmts.into_iter().next() else {
            unreachable!()
        };

        expr
    } else {
        syn::Expr::Block(ExprBlock {
            attrs: Vec::new(),
            label: None,
            block,
        })
    };

    match expr {
        syn::Expr::Async(_) | syn::Expr::Closure(_) => expr,

        _ => {
            parse_quote_spanned! {expr.span()=>
                (move || #expr)()
            }
        }
    }
}
