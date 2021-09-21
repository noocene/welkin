use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{FromWelkin, ToWelkin};

#[derive(Clone, Debug)]
pub struct BoxPoly<T>(pub T);

#[derive(Debug)]
pub enum InvalidBoxPoly<T: FromWelkin> {
    InvalidSized,
    Contents(T::Error),
}

impl<T: FromWelkin> FromWelkin for BoxPoly<T> {
    type Error = InvalidBoxPoly<T>;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Apply { argument, .. } = body.into_inner() {
                Ok(BoxPoly(
                    T::from_welkin(argument.into_inner()).map_err(InvalidBoxPoly::Contents)?,
                ))
            } else {
                Err(InvalidBoxPoly::InvalidSized)
            }
        } else {
            Err(InvalidBoxPoly::InvalidSized)
        }
    }
}

impl<T: ToWelkin> ToWelkin for BoxPoly<T> {
    type Error = T::Error;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        let mut term = Term::Variable(Index(0));

        term = Term::Apply {
            function: Box::new(term),
            argument: Box::new(self.0.to_welkin()?),
            erased: false,
        };

        term = Term::Lambda {
            erased: false,
            body: Box::new(term),
        };

        Ok(term)
    }
}
