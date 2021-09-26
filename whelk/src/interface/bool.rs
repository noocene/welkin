use std::convert::Infallible;

use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{FromWelkin, ToWelkin};

#[derive(Clone, Copy, Debug)]
pub struct Bool(pub bool);

#[derive(Clone, Copy, Debug)]
pub struct InvalidBool;

impl ToWelkin for Bool {
    type Error = Infallible;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        Ok(Term::Lambda {
            erased: false,
            body: Box::new(Term::Lambda {
                erased: false,
                body: Box::new(Term::Variable(Index(if self.0 { 1 } else { 0 }))),
            }),
        })
    }
}

impl FromWelkin for Bool {
    type Error = InvalidBool;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Lambda { body, .. } = body.into_inner() {
                match body.into_inner() {
                    Term::Variable(Index(0)) => return Ok(Bool(false)),
                    Term::Variable(Index(1)) => return Ok(Bool(true)),
                    _ => {}
                }
            }
        }
        Err(InvalidBool)
    }
}
