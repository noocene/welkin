use std::convert::Infallible;

use welkin_core::term::{Index, Term};

use super::{FromWelkin, ToWelkin};

#[derive(Clone, Copy, Debug)]
pub struct Unit;

impl ToWelkin for Unit {
    type Error = Infallible;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        Ok(Term::Lambda {
            erased: false,
            body: Box::new(Term::Variable(Index(0))),
        })
    }
}

impl FromWelkin for Unit {
    type Error = Infallible;

    fn from_welkin(_: Term<String>) -> Result<Self, Self::Error> {
        Ok(Unit)
    }
}
