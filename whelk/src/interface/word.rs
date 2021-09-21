use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{FromWelkin, ToWelkin};

#[derive(Clone, Debug)]
pub struct Word(pub Vec<bool>);

#[derive(Debug)]
pub struct InvalidWord;

impl FromWelkin for Word {
    type Error = InvalidWord;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        let mut data = vec![];
        let mut term = term;
        loop {
            while let Term::Lambda { body, .. } = term {
                term = body.into_inner();
            }
            match term {
                Term::Variable(_) => break Ok(Word(data)),
                Term::Apply {
                    argument, function, ..
                } => {
                    match function.into_inner() {
                        Term::Variable(Index(0)) => data.push(true),
                        Term::Variable(Index(1)) => data.push(false),
                        _ => break Err(InvalidWord),
                    };
                    term = argument.into_inner();
                }
                _ => break Err(InvalidWord),
            }
        }
    }
}

impl ToWelkin for Word {
    type Error = InvalidWord;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        let mut term = Term::Variable(Index(2));
        for _ in 0..3 {
            term = Term::Lambda {
                erased: false,
                body: Box::new(term),
            };
        }
        for bit in self.0 {
            term = Term::Apply {
                erased: false,
                argument: Box::new(term),
                function: Box::new(Term::Variable(Index(if bit { 0 } else { 1 }))),
            };
            for _ in 0..3 {
                term = Term::Lambda {
                    erased: false,
                    body: Box::new(term),
                };
            }
        }
        Ok(term)
    }
}
