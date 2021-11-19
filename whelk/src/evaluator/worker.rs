use futures::Future;
use welkin_core::term::Term;

use crate::{edit::zipper::analysis::NormalizationError, worker::WorkerWrapper};

use std::{fmt::Debug, pin::Pin};

use super::CoreEvaluator;
use thiserror::Error;

pub struct WorkerEvaluator(pub WorkerWrapper);

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct WorkerEvaluatorError(NormalizationError);

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct CoreWorkerEvaluatorError(welkin_core::term::NormalizationError);

impl CoreEvaluator for WorkerEvaluator {
    type Future = Pin<Box<dyn Future<Output = Result<Term<String>, Self::Error>>>>;
    type Error = CoreWorkerEvaluatorError;

    fn evaluate(&self, mut term: Term<String>) -> Self::Future {
        let worker = self.0.clone();
        Box::pin(async move { Ok(worker.evaluate(term).await) })
    }
}
