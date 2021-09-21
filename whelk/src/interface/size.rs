use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{FromWelkin, ToWelkin};

#[derive(Clone, Debug)]
pub struct Size(pub usize);

#[derive(Debug)]
pub struct InvalidSize;

impl FromWelkin for Size {
    type Error = InvalidSize;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Lambda { body, .. } = body.into_inner() {
                let mut term = body;
                let mut ctr = 0;
                while let Term::Apply { argument, .. } = term.into_inner() {
                    ctr += 1;
                    term = argument;
                }
                return Ok(Size(ctr));
            }
        }
        Err(InvalidSize)
    }
}

impl ToWelkin for Size {
    type Error = InvalidSize;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        let mut term = Term::Variable(Index(0));
        for _ in 0..self.0 {
            term = Term::Apply {
                erased: false,
                argument: Box::new(term),
                function: Box::new(Term::Variable(Index(1))),
            };
        }
        for _ in 0..2 {
            term = Term::Lambda {
                erased: false,
                body: Box::new(term),
            };
        }
        Ok(term)
    }
}
