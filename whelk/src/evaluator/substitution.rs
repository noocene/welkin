use welkin_core::term::{DefinitionResult, Definitions, Term};

use crate::edit::{
    dynamic::abst::controls::Zero,
    zipper::analysis::{AnalysisTerm, NormalizationError, TypedDefinitions},
};

use std::fmt::Debug;

use super::{CoreEvaluator, Evaluator};
use thiserror::Error;

#[derive(Clone)]
pub struct NullDefinitions;

impl Definitions<String> for NullDefinitions {
    fn get(&self, _: &String) -> Option<DefinitionResult<Term<String>>> {
        None
    }
}

pub struct Substitution<T>(pub T);

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct SubstitutionError(NormalizationError);

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct CoreSubstitutionError(welkin_core::term::NormalizationError);

impl<U: Zero + Debug + Clone, T: TypedDefinitions<U>> Evaluator<U> for Substitution<T> {
    type Error = SubstitutionError;

    fn evaluate(&self, mut term: AnalysisTerm<U>) -> Result<AnalysisTerm<U>, Self::Error> {
        term.normalize_in(&self.0).map_err(SubstitutionError)?;
        Ok(term)
    }
}

impl<T: Definitions<String>> CoreEvaluator for Substitution<T> {
    type Error = CoreSubstitutionError;

    fn evaluate(&self, mut term: Term<String>) -> Result<Term<String>, Self::Error> {
        term.normalize(&self.0).map_err(CoreSubstitutionError)?;
        Ok(term)
    }
}
