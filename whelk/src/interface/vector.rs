use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{FromWelkin, ToWelkin};

#[derive(Clone, Debug)]
pub struct Vector<T>(pub Vec<T>);

#[derive(Debug)]
pub enum InvalidVector<T: FromWelkin> {
    InvalidVector,
    Contents(T::Error),
}

impl<T: FromWelkin> FromWelkin for Vector<T> {
    type Error = InvalidVector<T>;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        let mut data = vec![];
        let mut term = term;
        loop {
            while let Term::Lambda { body, .. } = term {
                term = body.into_inner();
            }
            match term {
                Term::Variable(_) => break Ok(Vector(data)),
                Term::Apply {
                    argument, function, ..
                } => {
                    if let Term::Apply { argument, .. } = function.into_inner() {
                        data.push(
                            T::from_welkin(argument.into_inner())
                                .map_err(InvalidVector::Contents)?,
                        );
                    } else {
                        break Err(InvalidVector::InvalidVector);
                    }
                    term = argument.into_inner();
                }
                _ => break Err(InvalidVector::InvalidVector),
            }
        }
    }
}

impl<T: ToWelkin> ToWelkin for Vector<T> {
    type Error = T::Error;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        let mut term = Term::Variable(Index(1));

        for _ in 0..2 {
            term = Term::Lambda {
                erased: false,
                body: Box::new(term),
            };
        }

        for element in self.0 {
            term = Term::Apply {
                argument: Box::new(term),
                erased: false,
                function: Box::new(Term::Apply {
                    erased: false,
                    function: Box::new(Term::Variable(Index(0))),
                    argument: Box::new(element.to_welkin()?),
                }),
            };

            for _ in 0..2 {
                term = Term::Lambda {
                    erased: false,
                    body: Box::new(term),
                };
            }
        }

        Ok(term)
    }
}
