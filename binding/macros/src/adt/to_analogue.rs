use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Ident, Token, WhereClause};
use synstructure::Structure;

use crate::adt::is_inductive;

pub fn derive(structure: &Structure) -> TokenStream {
    let mut where_clause: WhereClause = parse_quote!(where);

    let mut inductive_impls = quote! {};

    for variant in structure.variants() {
        for binding in variant.bindings() {
            let ty = &binding.ast().ty;

            if !is_inductive(binding) {
                where_clause.predicates.push(parse_quote! {
                    #ty: ToAnalogue
                });
            } else {
                let mut where_clause: WhereClause = parse_quote!(where);

                let generics: Punctuated<Ident, Token![,]> = binding
                    .referenced_ty_params()
                    .into_iter()
                    .cloned()
                    .collect();

                for parameter in binding.referenced_ty_params() {
                    where_clause.predicates.push(parse_quote! {
                        #parameter: ToAnalogue
                    });
                }

                inductive_impls = quote! {
                    #inductive_impls

                    impl<#generics> ToAnalogue for #ty #where_clause {
                        type Analogue = <<Self as Wrapper>::Inner as ToAnalogue>::Analogue;

                        fn to_analogue(self) -> <Self as ToAnalogue>::Analogue {
                            ToAnalogue::to_analogue(*self)
                        }
                    }
                };
            }
        }
    }

    let inductive_uses = if inductive_impls.is_empty() {
        quote! {}
    } else {
        quote! {
            use ::std::boxed::Box;
            use welkin_binding::Wrapper;
        }
    };

    structure.gen_impl(quote! {
        extern crate welkin_binding;
        use welkin_binding::ToAnalogue;

        #inductive_uses

        #inductive_impls

        gen impl ToAnalogue for @Self #where_clause {
            type Analogue = Self;

            fn to_analogue(self) -> Self {
                self
            }
        }
    })
}
