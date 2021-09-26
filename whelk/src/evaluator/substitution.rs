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

pub struct Substitution;

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct SubstitutionError(NormalizationError);

impl Evaluator for Substitution {
    type Error = SubstitutionError;

    fn evaluate(&self, mut term: Term<String>) -> Result<Term<String>, Self::Error> {
        term.normalize(&NullDefinitions)
            .map_err(SubstitutionError)?;
        Ok(term)
    }
}
