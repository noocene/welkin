use welkin_core::term::Term;
mod substitution;
pub use substitution::Substitution;

pub trait Evaluator {
    type Error;

    fn evaluate(&self, term: Term<String>) -> Result<Term<String>, Self::Error>;
}
