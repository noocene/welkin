use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{AddBounds, BindStyle, Structure};

mod analogous;
mod from_analogue;
mod from_welkin;
mod to_analogue;
mod to_welkin;

pub fn derive(mut structure: Structure) -> TokenStream {
    structure.add_bounds(AddBounds::None);
    structure.bind_with(|_| BindStyle::Move);

    let analogous_impl = analogous::derive(&structure);

    let to_analogue_impl = to_analogue::derive(&structure);
    let from_analogue_impl = from_analogue::derive(&structure);

    let to_welkin_impl = to_welkin::derive(&structure);
    let from_welkin_impl = from_welkin::derive(&structure);

    quote! {
        #analogous_impl

        #to_analogue_impl
        #from_analogue_impl

        #to_welkin_impl
        #from_welkin_impl
    }
}
