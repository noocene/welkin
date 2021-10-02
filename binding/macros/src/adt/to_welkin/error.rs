use std::{collections::HashSet, iter::repeat};

use proc_macro2::Ident;
use quote::format_ident;
use syn::{parse_quote, ItemEnum};
use synstructure::Structure;

use crate::adt::is_inductive;

pub fn derive(structure: &Structure) -> (ItemEnum, Ident, Vec<Ident>) {
    let vis = &structure.ast().vis;

    let to_welkin_error_ident = format_ident!("{}ToWelkinError", structure.ast().ident);

    let mut to_welkin_error: ItemEnum = parse_quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Error)]
        #vis enum #to_welkin_error_ident {}
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
        to_welkin_error.generics.params.push(parse_quote! {
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

        to_welkin_error.variants.push(parse_quote! {
            #variant_ident(#ident)
        });
    }

    error_variant_idents.reverse();

    (to_welkin_error, to_welkin_error_ident, error_variant_idents)
}
