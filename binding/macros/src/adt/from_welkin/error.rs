use std::{collections::HashSet, iter::repeat};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, ItemEnum};
use synstructure::{AddBounds, Structure};

use crate::adt::is_inductive;

pub fn derive(structure: &Structure) -> (TokenStream, Ident, Vec<Ident>) {
    let vis = &structure.ast().vis;
    let ident = &structure.ast().ident;

    let from_welkin_error_ident = format_ident!("{}FromWelkinError", structure.ast().ident);

    let mut from_welkin_error: ItemEnum = parse_quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Error, Clone)]
        #vis enum #from_welkin_error_ident {}
    };

    let mut binding_names = HashSet::new();
    let mut duplicate_binding_names = HashSet::new();

    for binding in structure.variants().iter().flat_map(|variant| {
        variant
            .bindings()
            .iter()
            .filter(|binding| !is_inductive(binding))
    }) {
        if let Some((true, binding_name)) = binding.ast().ident.as_ref().map(|ident| {
            let name = format!("{}", ident);
            (binding_names.contains(&name), name)
        }) {
            duplicate_binding_names.insert(binding_name);
        } else if let Some(ident) = binding.ast().ident.as_ref() {
            binding_names.insert(format!("{}", ident));
        }
    }

    let mut error_variant_idents = vec![];

    for (idx, ((binding_idx, binding), variant)) in structure
        .variants()
        .iter()
        .flat_map(|variant| {
            variant
                .bindings()
                .iter()
                .filter(|binding| !is_inductive(binding))
                .enumerate()
                .zip(repeat(variant))
        })
        .enumerate()
    {
        let ident = format_ident!("T{}", idx);
        from_welkin_error.generics.params.push(parse_quote! {
            #ident
        });

        let variant_ident = if binding
            .ast()
            .ident
            .as_ref()
            .map(|ident| duplicate_binding_names.contains(&format!("{}", ident)))
            .unwrap_or(true)
        {
            format_ident!(
                "{}{}",
                variant.ast().ident,
                binding
                    .ast()
                    .ident
                    .as_ref()
                    .unwrap_or(&format_ident!("Field{}", binding_idx))
            )
        } else {
            format_ident!(
                "{}",
                binding
                    .ast()
                    .ident
                    .as_ref()
                    .unwrap_or(&format_ident!("Field{}", binding_idx))
            )
        };

        error_variant_idents.push(variant_ident.clone());

        from_welkin_error.variants.push(parse_quote! {
            #variant_ident(#ident)
        });
    }

    error_variant_idents.reverse();

    let error_ast = parse_quote!(#from_welkin_error);
    let mut error_structure = Structure::new(&error_ast);

    error_structure.add_bounds(AddBounds::Generics);

    let arms = error_structure.each_variant(|variant| {
        let format_string = format!(
            "error reading field \"{}\" of type \"{}\": {{}}",
            variant.ast().ident,
            ident
        );
        quote! {
            write!(f, #format_string, __binding_0)?;
        }
    });

    from_welkin_error.variants.push(parse_quote! {
        ExpectedLambda(Term<::std::string::String>)
    });
    from_welkin_error.variants.push(parse_quote! {
        ExpectedApplyOrVariable(Term<::std::string::String>)
    });
    from_welkin_error.variants.push(parse_quote! {
        InvalidVariant {
            got: usize,
            expected_at_most: usize
        }
    });
    from_welkin_error.variants.push(parse_quote! {
        InsufficientFields
    });

    let display_impl = error_structure.gen_impl(quote! {
        use ::std::fmt::{self, Formatter, Display};

        gen impl Display for @Self {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    #arms
                    #from_welkin_error_ident::ExpectedLambda(got) => {
                        write!(f, "expected lambda, got {:?}", got)?;
                    }
                    #from_welkin_error_ident::ExpectedApplyOrVariable(got) => {
                        write!(f, "expected apply or variable, got {:?}", got)?;
                    }
                    #from_welkin_error_ident::InvalidVariant { got, expected_at_most } => {
                        write!(f, "invalid variant {}, expected at most {}", got, expected_at_most)?;
                    }
                    #from_welkin_error_ident::InsufficientFields => {
                        write!(f, "insufficient fields present in term")?;
                    }
                }

                Ok(())
            }
        }
    });

    (
        quote! {
            #from_welkin_error
            #display_impl
        },
        from_welkin_error_ident,
        error_variant_idents,
    )
}
