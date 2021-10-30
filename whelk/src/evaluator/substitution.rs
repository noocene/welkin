use welkin_core::term::{DefinitionResult, Definitions, NormalizationError, Term};

use super::Evaluator;
use thiserror::Error;

#[derive(Clone)]
pub struct NullDefinitions;

impl Definitions<String> for NullDefinitions {
    fn get(&self, _: &String) -> Option<DefinitionResult<Term<String>>> {
        None
    }
}

pub struct Substitution<T: Definitions<String>>(pub T);

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct SubstitutionError(NormalizationError);

impl<T: Definitions<String>> Evaluator for Substitution<T> {
    type Error = SubstitutionError;

    fn evaluate(&self, mut term: Term<String>) -> Result<Term<String>, Self::Error> {
        term.normalize(&self.0).map_err(SubstitutionError)?;
        Ok(term)
    }
}
