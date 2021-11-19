use welkin_core::{
    net::{Index, Net, VisitNetExt},
    term::{DefinitionResult, Definitions, Term},
};

use crate::edit::zipper::analysis::NormalizationError;

use std::fmt::Debug;

use super::CoreEvaluator;
use thiserror::Error;

#[derive(Clone)]
pub struct NullDefinitions;

impl Definitions<String> for NullDefinitions {
    fn get(&self, _: &String) -> Option<DefinitionResult<Term<String>>> {
        None
    }
}

pub struct Inet<T>(pub T);

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct InetError(NormalizationError);

#[derive(Debug, Error)]
#[error("substitution error")]
pub struct CoreInetError(welkin_core::term::NormalizationError);

impl<T: Definitions<String>> CoreEvaluator for Inet<T> {
    type Error = CoreInetError;

    fn evaluate(&self, mut term: Term<String>) -> Result<Term<String>, Self::Error> {
        let mut net = term
            .stratified(&self.0)
            .unwrap()
            .into_net::<Net<u32>>()
            .unwrap();
        net.reduce_all();
        Ok(net.read_term(Index(0)))
    }
}
