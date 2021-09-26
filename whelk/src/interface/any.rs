use std::convert::Infallible;

use welkin_core::term::Term;

use super::{FromWelkin, ToWelkin};

#[derive(Clone, Debug)]
pub struct Any(pub Term<String>);

impl FromWelkin for Any {
    type Error = Infallible;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error> {
        Ok(Any(term))
    }
}

impl ToWelkin for Any {
    type Error = Infallible;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        Ok(self.0)
    }
}
