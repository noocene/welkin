pub mod iter;

use super::w;
use derivative::Derivative;
use std::marker::PhantomData;
use thiserror::Error;
use welkin_binding::{FromAnalogue, FromWelkin};

use welkin_core::term::Term;

use crate::evaluator::CoreEvaluator;

#[derive(Clone, Debug)]
pub struct IoRequest<G, T> {
    request: G,
    term: Term<String>,
    phantom: PhantomData<T>,
}

#[derive(Derivative, Error)]
#[derivative(Debug(
    bound = "<<G as FromAnalogue>::Analogue as FromWelkin>::Error: std::fmt::Debug, <<T as FromAnalogue>::Analogue as FromWelkin>::Error: std::fmt::Debug, E::Error: std::fmt::Debug"
))]
#[error("fulfillment error")]
pub enum FulfillmentError<G: FromAnalogue, T: FromAnalogue, E: CoreEvaluator> {
    Evaluator(E::Error),
    Io(<w::IO<G, T> as FromWelkin>::Error),
}

impl<G: FromAnalogue, T: FromAnalogue> IoRequest<G, T> {
    pub fn fulfill<E: CoreEvaluator>(
        self,
        response: Term<String>,
        evaluator: &E,
    ) -> Result<w::IO<G, T>, FulfillmentError<G, T, E>> {
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

impl<G, T> w::IO<G, T> {
    pub fn into_request(self) -> Option<IoRequest<G, T>> {
        match self {
            w::IO::end { .. } => None,
            w::IO::call { request, then } => Some(IoRequest {
                request,
                term: then.0,
                phantom: PhantomData,
            }),
        }
    }
}
