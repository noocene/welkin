pub use macros::Adt;
pub use thiserror::Error;
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

pub trait Adt: ToWelkin + FromWelkin {
    type DefinitionType: ToWelkin;
    type Definition: ToWelkin;

    type Constructor: ToWelkin;
    type ConstructorType: ToWelkin;
    type Constructors: Iterator<Item = (Self::ConstructorType, Self::Constructor)>;

    fn definition() -> (Self::DefinitionType, Self::Definition);
    fn constructors() -> Self::Constructors;
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
