use welkin_core::term::{Index, Term};

use crate::bindings::w;

impl From<usize> for w::Size {
    fn from(data: usize) -> Self {
        let mut term = Term::Variable(Index(0));
        for _ in 0..data {
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
        w::Size(term)
    }
}
