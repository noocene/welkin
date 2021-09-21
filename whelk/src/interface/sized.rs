use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{size::InvalidSize, FromWelkin, Size, ToWelkin, WString, Word};

#[derive(Clone, Debug)]
pub struct WSized<T: Length>(pub T);

#[derive(Debug)]
pub enum InvalidSized<T: FromWelkin> {
    InvalidSized,
    Contents(T::Error),
}

pub trait Length {
    fn len(&self) -> usize;
}

impl Length for WString {
    fn len(&self) -> usize {
        String::len(&self.0)
    }
}

impl Length for Word {
    fn len(&self) -> usize {
        Vec::len(&self.0)
    }
}

impl<T: FromWelkin + Length> FromWelkin for WSized<T> {
    type Error = InvalidSized<T>;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Apply { argument, .. } = body.into_inner() {
                Ok(WSized(
                    T::from_welkin(argument.into_inner()).map_err(InvalidSized::Contents)?,
                ))
            } else {
                Err(InvalidSized::InvalidSized)
            }
        } else {
            Err(InvalidSized::InvalidSized)
        }
    }
}

#[derive(Debug)]
pub enum SizedToWelkinError<T: ToWelkin> {
    Content(T::Error),
    Size(InvalidSize),
}

impl<T: ToWelkin + Length> ToWelkin for WSized<T> {
    type Error = SizedToWelkinError<T>;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        let mut term = Term::Variable(Index(0));

        term = Term::Apply {
            function: Box::new(term),
            argument: Box::new(
                Size(self.0.len())
                    .to_welkin()
                    .map_err(SizedToWelkinError::Size)?,
            ),
            erased: false,
        };

        term = Term::Apply {
            function: Box::new(term),
            argument: Box::new(self.0.to_welkin().map_err(SizedToWelkinError::Content)?),
            erased: false,
        };

        term = Term::Lambda {
            erased: false,
            body: Box::new(term),
        };

        Ok(term)
    }
}
