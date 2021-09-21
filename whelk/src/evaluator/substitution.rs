use welkin_core::term::{DefinitionResult, Definitions, NormalizationError, Term};

use super::Evaluator;

#[derive(Clone)]
pub struct NullDefinitions;

impl Definitions<String> for NullDefinitions {
    fn get(&self, _: &String) -> Option<DefinitionResult<Term<String>>> {
        None
    }
}

pub struct Substitution;

impl Evaluator for Substitution {
    type Error = NormalizationError;

    fn evaluate(&self, mut term: Term<String>) -> Result<Term<String>, Self::Error> {
        term.normalize(&NullDefinitions)?;
        Ok(term)
    }
}
