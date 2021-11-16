use std::pin::Pin;

use crate::{bindings::w, evaluator::CoreEvaluator};
use futures::Future;
use thiserror::Error;
use welkin_binding::{FromWelkin, ToWelkin};
use welkin_core::term::Term;

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
    Bool(<w::Bool as FromWelkin>::Error),
}

#[derive(Debug, Clone, Error)]
pub enum StepError<E, T> {
    #[error("step error in evaluator: {0}")]
    Evaluator(E),
    #[error("step error in fulfillment: {0}")]
    Fulfill(T),
    #[error("step error in io read: {0:?}")]
    Io(<w::BoxPoly<w::WhelkIO<w::Any>> as FromWelkin>::Error),
}

impl LoopRequest {
    pub fn new(state: Term<String>, predicate: Term<String>, step: Term<String>) -> Self {
        LoopRequest {
            state,
            predicate,
            step,
        }
    }

    pub fn proceed<E: CoreEvaluator>(&self, evaluator: &E) -> Result<bool, ProceedError<E::Error>> {
        evaluator
            .evaluate(Term::Apply {
                erased: false,
                argument: Box::new(self.state.clone()),
                function: Box::new(self.predicate.clone()),
            })
            .map_err(ProceedError::Evaluator)
            .and_then(|term| {
                w::Bool::from_welkin(term)
                    .map(|a| match a {
                        w::Bool::r#true => true,
                        w::Bool::r#false => false,
                    })
                    .map_err(ProceedError::Bool)
            })
    }
    pub fn into_state(self) -> Term<String> {
        self.state
    }
    pub fn step<
        'a,
        E: CoreEvaluator,
        T,
        Fut: Future<Output = Result<w::Any, T>>,
        F: FnOnce(w::WhelkIO<w::Any>) -> Fut + 'a,
    >(
        &'a mut self,
        evaluator: &'a E,
        fulfill: F,
    ) -> Pin<Box<dyn Future<Output = Result<(), StepError<E::Error, T>>> + 'a>> {
        Box::pin(async move {
            let io = match w::BoxPoly::<w::WhelkIO<w::Any>>::from_welkin(
                evaluator
                    .evaluate(Term::Apply {
                        erased: false,
                        argument: Box::new(
                            w::BoxPoly::new {
                                data: w::Any(self.state.clone()),
                            }
                            .to_welkin()
                            .unwrap(),
                        ),
                        function: Box::new(self.step.clone()),
                    })
                    .map_err(StepError::Evaluator)?,
            )
            .map_err(StepError::Io)?
            {
                w::BoxPoly::new { data } => data,
            };

            self.state = fulfill(io).await.map_err(StepError::Fulfill)?.0;

            Ok(())
        })
    }
}
