mod substitution;
pub use substitution::Substitution;
mod inet;
pub use inet::Inet;
use welkin_core::term::Term;

use crate::edit::zipper::analysis::AnalysisTerm;

pub trait Evaluator<T> {
    type Error;

    fn evaluate(&self, term: AnalysisTerm<T>) -> Result<AnalysisTerm<T>, Self::Error>;
}

pub trait CoreEvaluator {
    type Error;

    fn evaluate(&self, term: Term<String>) -> Result<Term<String>, Self::Error>;
}
