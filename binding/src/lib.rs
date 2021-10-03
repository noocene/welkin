use std::marker::PhantomData;

use bumpalo::Bump;
pub use macros::Adt;
pub use thiserror::Error;
use welkin::{
    compiler::{item::Compile, LocalResolver},
    parser::{self, AbsolutePath, BumpBox, BumpVec, Data, Ident, Path, Variant},
};
pub use welkin_core;
use welkin_core::term::Term;

pub trait FromWelkin: Sized {
    type Error;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error>;
}

pub trait ToWelkin {
    type Error;

    fn to_welkin(self) -> Result<Term<String>, Self::Error>;
}

#[derive(Clone)]
pub enum AdtConstructor {
    Inductive,
    Other(&'static str),
}

#[derive(Clone)]

pub enum Type {
    Parameter(usize),
    Data {
        constructor: AdtConstructor,
        params: &'static [Type],
    },
}

impl Type {
    fn generate<'a>(self, bump: &'a Bump, this: &str) -> parser::Term<'a> {
        match self {
            Type::Parameter(idx) => parser::Term::Reference(Path(BumpVec::unary_in(
                Ident::from_str(&format!("T{}", idx), &bump),
                &bump,
            ))),
            Type::Data {
                constructor,
                params,
            } => parser::Term::Application {
                erased: true,
                function: match constructor {
                    AdtConstructor::Inductive => BumpBox::new_in(
                        parser::Term::Reference(Path(BumpVec::unary_in(
                            Ident::from_str(this, &bump),
                            &bump,
                        ))),
                        bump,
                    ),
                    AdtConstructor::Other(name) => BumpBox::new_in(
                        parser::Term::Reference(Path(BumpVec::unary_in(
                            Ident::from_str(name, &bump),
                            &bump,
                        ))),
                        bump,
                    ),
                },
                arguments: BumpVec::from_iterator(
                    params.iter().map(|ty| ty.clone().generate(&bump, this)),
                    &bump,
                ),
            },
        }
    }
}

pub struct AdtVariant {
    pub fields: &'static [Type],
    pub name: &'static str,
}

pub struct AdtDefinition {
    pub variants: &'static [AdtVariant],
    pub name: &'static str,
    pub params: usize,
}

#[derive(Debug)]
pub struct Definition {
    pub path: AbsolutePath,
    pub ty: Term<AbsolutePath>,
    pub term: Term<AbsolutePath>,
}

mod typed_sealed {
    use crate::{Adt, Analogous, Dummy};

    pub trait Sealed {}
    impl<T: Analogous> Sealed for T where T::Analogue: Adt {}
    impl<const INDEX: usize> Sealed for Dummy<INDEX> {}
}

pub trait Typed: typed_sealed::Sealed {
    const TYPE: Type;
}

impl<T: Analogous> Typed for T
where
    T::Analogue: Adt,
{
    const TYPE: Type = Type::Data {
        constructor: AdtConstructor::Other(<<T as Analogous>::Analogue as Adt>::DEFINITION.name),
        params: <<T as Analogous>::Analogue as Adt>::PARAMS,
    };
}

pub struct Dummy<const INDEX: usize>(PhantomData<[(); INDEX]>);

impl<const INDEX: usize> Typed for Dummy<INDEX> {
    const TYPE: Type = Type::Parameter(INDEX);
}

impl AdtDefinition {
    pub fn generate(self) -> Vec<Definition> {
        let bump = Bump::new();

        let data = Data {
            variants: BumpVec::from_iterator(
                self.variants.iter().map(|variant| Variant {
                    ident: Ident::from_str(variant.name, &bump),
                    inhabitants: BumpVec::from_iterator(
                        variant.fields.iter().map(|ty| {
                            (
                                Ident::from_str("_", &bump),
                                ty.clone().generate(&bump, self.name),
                                false,
                            )
                        }),
                        &bump,
                    ),
                    indices: BumpVec::new_in(&bump),
                }),
                &bump,
            ),
            type_arguments: BumpVec::from_iterator(
                (0..self.params)
                    .map(|idx| (Ident::from_str(&format!("T{}", idx), &bump), None, true)),
                &bump,
            ),
            indices: BumpVec::new_in(&bump),
            ident: Ident::from_str(self.name, &bump),
        };

        data.compile(LocalResolver::new())
            .into_iter()
            .map(|(path, ty, term)| Definition { path, ty, term })
            .collect()
    }
}

pub trait Adt {
    const DEFINITION: AdtDefinition;
    const PARAMS: &'static [Type];
}

pub trait Analogous {
    type Analogue;
}

pub trait FromAnalogue: Analogous<Analogue = <Self as FromAnalogue>::Analogue> {
    type Analogue: FromWelkin;

    fn from_analogue(analogue: <Self as FromAnalogue>::Analogue) -> Self;
}

pub trait ToAnalogue: Analogous<Analogue = <Self as ToAnalogue>::Analogue> {
    type Analogue: ToWelkin;

    fn to_analogue(self) -> <Self as ToAnalogue>::Analogue;
}

impl<T: Analogous> Analogous for Box<T> {
    type Analogue = T::Analogue;
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for Box<T> {}
}

pub trait Wrapper: sealed::Sealed {
    type Inner;
}

impl<T> Wrapper for Box<T> {
    type Inner = T;
}
