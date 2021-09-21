use welkin_core::term::{alloc::IntoInner, Term};

use crate::interface::{box_poly::InvalidBoxPoly, whelk, BoxPoly, FromWelkin, Io, Unit};

pub struct Whelk(pub BoxPoly<Io<whelk::Request, Unit>>);

#[derive(Debug)]
pub enum InvalidWhelk {
    InvalidWhelk,
    Io(InvalidBoxPoly<Io<whelk::Request, Unit>>),
}

impl FromWelkin for Whelk {
    type Error = InvalidWhelk;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Apply { argument, .. } = body.into_inner() {
                Ok(Whelk(
                    FromWelkin::from_welkin(argument.into_inner()).map_err(InvalidWhelk::Io)?,
                ))
            } else {
                Err(InvalidWhelk::InvalidWhelk)
            }
        } else {
            Err(InvalidWhelk::InvalidWhelk)
        }
    }
}
