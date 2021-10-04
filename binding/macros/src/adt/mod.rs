use std::error::Error;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Field};
use synstructure::{AddBounds, BindStyle, BindingInfo, Structure};

mod analogous;
mod from_analogue;
mod from_welkin;
mod to_analogue;
mod to_welkin;

mod derive;

pub fn is_inductive(binding: &BindingInfo) -> bool {
    is_field_inductive(binding.ast())
}

pub fn is_field_inductive(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path == parse_quote!(inductive))
}

pub fn derive(mut structure: Structure) -> Result<TokenStream, Box<dyn Error>> {
    structure.add_bounds(AddBounds::None);
    structure.bind_with(|_| BindStyle::Move);

    let analogous_impl = analogous::derive(&structure);

    let to_analogue_impl = to_analogue::derive(&structure);
    let from_analogue_impl = from_analogue::derive(&structure);

    let to_welkin_impl = to_welkin::derive(&structure);
    let from_welkin_impl = from_welkin::derive(&structure);

    let adt_impl = derive::derive(&structure);

    Ok(quote! {
        #analogous_impl

        #to_analogue_impl
        #from_analogue_impl

        #to_welkin_impl
        #from_welkin_impl

        #adt_impl

    })
}
