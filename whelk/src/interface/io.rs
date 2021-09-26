use std::marker::PhantomData;

use welkin_core::term::{alloc::IntoInner, Index, Term};

use crate::evaluator::Evaluator;

use super::FromWelkin;

use derivative::Derivative;
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Io<G, T> {
    Data(T),
    Request(IoRequest<G, T>),
}

#[derive(Clone, Debug)]
pub struct IoRequest<G, T> {
    request: G,
    term: Term<String>,
    phantom: PhantomData<T>,
}

#[derive(Derivative, Clone)]
#[derivative(Debug(bound = "G::Error: std::fmt::Debug, T::Error: std::fmt::Debug"))]
pub enum InvalidIo<G: FromWelkin, T: FromWelkin> {
    InvalidIo,
    Data(T::Error),
    Request(G::Error),
}

#[derive(Derivative, Error)]
#[derivative(Debug(
    bound = "G::Error: std::fmt::Debug, T::Error: std::fmt::Debug, E::Error: std::fmt::Debug"
))]
#[error("fulfillment error")]
pub enum FulfillmentError<G: FromWelkin, T: FromWelkin, E: Evaluator> {
    Evaluator(E::Error),
    Io(InvalidIo<G, T>),
}

impl<G: FromWelkin, T: FromWelkin> IoRequest<G, T> {
    pub fn request(&self) -> &G {
        &self.request
    }

    pub fn fulfill<E: Evaluator>(
        self,
        response: Term<String>,
        evaluator: &E,
    ) -> Result<Io<G, T>, FulfillmentError<G, T, E>> {
        Ok(FromWelkin::from_welkin(
            evaluator
                .evaluate(Term::Apply {
                    erased: false,
                    argument: Box::new(response),
                    function: Box::new(self.term),
                })
                .map_err(FulfillmentError::Evaluator)?,
        )
        .map_err(FulfillmentError::Io)?)
    }
}

impl<T: FromWelkin, G: FromWelkin> FromWelkin for Io<G, T> {
    type Error = InvalidIo<G, T>;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error> {
        let mut term = term;
        loop {
            while let Term::Lambda { body, .. } = term {
                term = body.into_inner();
            }
            match term {
                Term::Apply {
                    argument, function, ..
                } => {
                    match function.into_inner() {
                        Term::Variable(Index(1)) => {
                            break T::from_welkin(argument.into_inner())
                                .map_err(InvalidIo::Data)
                                .map(|a| Io::Data(a))
                        }
                        Term::Apply {
                            function,
                            argument: b_argument,
                            ..
                        } => {
                            if let Term::Variable(Index(0)) = function.into_inner() {
                                break Ok(Io::Request(IoRequest {
                                    request: G::from_welkin(b_argument.into_inner())
                                        .map_err(InvalidIo::Request)?,
                                    phantom: PhantomData,
                                    term: argument.into_inner(),
                                }));
                            } else {
                                break Err(InvalidIo::InvalidIo);
                            }
                        }
                        _ => break Err(InvalidIo::InvalidIo),
                    };
                }
                _ => break Err(InvalidIo::InvalidIo),
            }
        }
    }
}
