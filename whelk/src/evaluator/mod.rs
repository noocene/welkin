mod substitution;
use futures::Future;
pub use substitution::Substitution;
mod inet;
pub use inet::Inet;
use welkin_core::term::Term;
mod worker;
pub use worker::WorkerEvaluator;

use crate::edit::zipper::analysis::AnalysisTerm;

pub trait Evaluator<T> {
    type Error;

    fn evaluate(&self, term: AnalysisTerm<T>) -> Result<AnalysisTerm<T>, Self::Error>;
}

pub trait CoreEvaluator {
    type Error;
    type Future: Future<Output = Result<Term<String>, Self::Error>>;

    fn evaluate(&self, term: Term<String>) -> Self::Future;
}
