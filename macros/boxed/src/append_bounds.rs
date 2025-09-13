use macro_helpers::error::EmitError;
use syn::Error;

use crate::utils::make_return_type;

use super::Args;

pub fn append_bounds(func: &mut syn::ItemFn, args: &Args) {
    if args.bounds.is_empty() {
        return;
    }

    let ty = make_return_type(&mut func.sig);
    let syn::Type::ImplTrait(impl_trait) = ty else {
        Error::new_spanned(
            ty,
            "`bounds` can only be used on `async fn`s and functions that return `impl Trait`",
        )
        .emit_error();
        return;
    };

    impl_trait.bounds.extend(args.bounds.clone());
}
