use std::pin::Pin;

use futures::Future;
use welkin_core::term::{alloc::IntoInner, Index, Term};

use thiserror::Error;

use crate::{
    evaluator::Evaluator,
    interface::{bool::InvalidBool, Any, Bool, BoxPoly, FromWelkin, Io, ToWelkin, WSized, WString},
};

#[derive(Debug, Clone)]
pub struct LoopRequest {
    state: Term<String>,
    predicate: Term<String>,
    step: Term<String>,
}

#[derive(Debug, Clone, Error)]
#[error("proceed error")]
pub enum ProceedError<E> {
    Evaluator(E),
    Bool(InvalidBool),
}

#[derive(Debug, Clone, Error)]
pub enum StepError<E, T> {
    #[error("step error in evaluator: {0}")]
    Evaluator(E),
    #[error("step error in fulfillment: {0}")]
    Fulfill(T),
    #[error("step error in io read: {0:?}")]
    Io(<BoxPoly<Io<Request, Any>> as FromWelkin>::Error),
}

impl LoopRequest {
    pub fn proceed<E: Evaluator>(&self, evaluator: &E) -> Result<bool, ProceedError<E::Error>> {
        evaluator
            .evaluate(Term::Apply {
                erased: false,
                argument: Box::new(self.state.clone()),
                function: Box::new(self.predicate.clone()),
            })
            .map_err(ProceedError::Evaluator)
            .and_then(|term| {
                Bool::from_welkin(term)
                    .map(|a| a.0)
                    .map_err(ProceedError::Bool)
            })
    }
    pub fn into_state(self) -> Term<String> {
        self.state
    }
    pub fn step<
        'a,
        E: Evaluator,
        T,
        Fut: Future<Output = Result<Any, T>>,
        F: FnOnce(Io<Request, Any>) -> Fut + 'a,
    >(
        &'a mut self,
        evaluator: &'a E,
        fulfill: F,
    ) -> Pin<Box<dyn Future<Output = Result<(), StepError<E::Error, T>>> + 'a>> {
        Box::pin(async move {
            let io = BoxPoly::<Io<Request, Any>>::from_welkin(
                evaluator
                    .evaluate(Term::Apply {
                        erased: false,
                        argument: Box::new(BoxPoly(Any(self.state.clone())).to_welkin().unwrap()),
                        function: Box::new(self.step.clone()),
                    })
                    .map_err(StepError::Evaluator)?,
            )
            .map_err(StepError::Io)?
            .0;

            self.state = fulfill(io).await.map_err(StepError::Fulfill)?.0;

            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub enum Request {
    Prompt,
    Print(WSized<WString>),
    Loop(LoopRequest),
}

#[derive(Debug, Clone)]
pub enum InvalidRequest {
    InvalidRequest,
    InvalidString(<WSized<WString> as FromWelkin>::Error),
}

impl FromWelkin for Request {
    type Error = InvalidRequest;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Lambda { body, .. } = body.into_inner() {
                if let Term::Lambda { body, .. } = body.into_inner() {
                    if let Term::Apply {
                        argument, function, ..
                    } = body.into_inner()
                    {
                        let o_argument = argument;
                        match function.into_inner() {
                            Term::Apply {
                                argument, function, ..
                            } => {
                                let predicate = argument.into_inner();
                                if let Term::Apply { argument, .. } = function.into_inner() {
                                    let state = argument.into_inner();
                                    let step = o_argument.into_inner();
                                    return Ok(Request::Loop(LoopRequest {
                                        predicate,
                                        step,
                                        state,
                                    }));
                                }
                            }
                            Term::Variable(Index(1)) => return Ok(Request::Prompt),
                            Term::Variable(Index(2)) => {
                                return Ok(Request::Print(
                                    FromWelkin::from_welkin(o_argument.into_inner())
                                        .map_err(InvalidRequest::InvalidString)?,
                                ))
                            }
                            _ => return Err(InvalidRequest::InvalidRequest),
                        }
                    }
                }
            }
        }
        Err(InvalidRequest::InvalidRequest)
    }
}
