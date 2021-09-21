use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{word::InvalidWord, Char, FromWelkin, ToWelkin, Vector};

#[derive(Clone, Debug)]
pub struct WString(pub String);

#[derive(Debug)]
pub enum InvalidString {
    InvalidVector(<Vector<Char> as FromWelkin>::Error),
    InvalidString,
}

impl From<<Vector<Char> as FromWelkin>::Error> for InvalidString {
    fn from(e: <Vector<Char> as FromWelkin>::Error) -> Self {
        InvalidString::InvalidVector(e)
    }
}

impl FromWelkin for WString {
    type Error = InvalidString;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Apply { argument, .. } = body.into_inner() {
                Ok(WString(
                    Vector::<Char>::from_welkin(argument.into_inner())?
                        .0
                        .into_iter()
                        .map(|a| a.0)
                        .collect(),
                ))
            } else {
                Err(InvalidString::InvalidString)
            }
        } else {
            Err(InvalidString::InvalidString)
        }
    }
}

impl From<WString> for String {
    fn from(ws: WString) -> Self {
        ws.0
    }
}

impl ToWelkin for WString {
    type Error = InvalidWord;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        Ok(Term::Lambda {
            erased: false,
            body: Box::new(Term::Apply {
                erased: false,
                function: Box::new(Term::Variable(Index(0))),
                argument: Box::new(
                    Vector::<Char>(self.0.chars().rev().map(Char).collect()).to_welkin()?,
                ),
            }),
        })
    }
}
