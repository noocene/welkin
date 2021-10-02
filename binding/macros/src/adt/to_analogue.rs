use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, WhereClause};
use synstructure::Structure;

pub fn derive(structure: &Structure) -> TokenStream {
    let mut where_clause: WhereClause = parse_quote!(where);

    for variant in structure.variants() {
        for binding in variant.bindings() {
            let ty = &binding.ast().ty;

            where_clause.predicates.push(parse_quote! {
                #ty: ToAnalogue
            });
        }
    }

    structure.gen_impl(quote! {
        extern crate welkin_binding;
        use welkin_binding::ToAnalogue;

        gen impl ToAnalogue for @Self #where_clause {
            type Analogue = Self;

            fn to_analogue(self) -> Self {
                self
            }
        }
    })
}
