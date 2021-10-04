use quote::quote;
use synstructure::decl_derive;
mod adt;

decl_derive!(
    [Adt, attributes(inductive)] =>
    adt_derive
);

fn adt_derive(item: synstructure::Structure) -> proc_macro2::TokenStream {
    adt::derive(item).unwrap_or_else(|e| {
        let e = format!("{}", e);

        quote! {
            const _: () = {
                compile_error!(#e);
                ()
            };
        }
    })
}
