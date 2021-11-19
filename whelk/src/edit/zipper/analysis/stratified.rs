use serde::{Deserialize, Serialize};

use super::{AnalysisTerm, Definitions};

#[derive(Debug, Serialize, Deserialize)]
pub enum StratificationError {
    MultiplicityMismatch,
    AffineUsedInBox,
    DupNonUnitBoxMultiplicity,
    RecursiveDefinition,
    UndefinedReference(String),
    ErasedUsed,
}

impl<T> AnalysisTerm<T> {
    fn uses(&self) -> usize {
        fn uses_helper<T>(term: &AnalysisTerm<T>, variable: usize) -> usize {
            use AnalysisTerm::*;
            match term {
                Variable(index, _) => {
                    if *index == variable {
                        1
                    } else {
                        0
                    }
                }
                Reference(_, _) | Function { .. } | Universe(_) | Hole(_) | Compressed(_) => 0,
                Lambda { body, erased, .. } => {
                    if *erased {
                        0
                    } else {
                        uses_helper(body, variable + 1)
                    }
                }
                Application {
                    function,
                    argument,
                    erased,
                    ..
                } => {
                    uses_helper(function, variable)
                        + if *erased {
                            0
                        } else {
                            uses_helper(argument, variable)
                        }
                }
                Put(term, _) => uses_helper(term, variable),
                Duplication {
                    expression, body, ..
                } => uses_helper(expression, variable) + uses_helper(body, variable + 1),

                Wrap(term, _) => uses_helper(term, variable),
                Annotation { term, ty, .. } => {
                    uses_helper(term, variable) + uses_helper(ty, variable)
                }
            }
        }

        uses_helper(self, 0)
    }

    fn is_boxed_n_times(&self, nestings: usize) -> bool {
        use AnalysisTerm::*;

        fn n_boxes_helper<T>(
            this: &AnalysisTerm<T>,
            variable: usize,
            nestings: usize,
            current_nestings: usize,
        ) -> bool {
            match this {
                Reference(_, _) | Universe(_) | Function { .. } | Hole(_) | Compressed(_) => true,
                Variable(index, _) => *index != variable || nestings == current_nestings,
                Lambda { body, .. } => {
                    n_boxes_helper(body, variable + 1, nestings, current_nestings)
                }
                Application {
                    function,
                    argument,
                    erased,
                    ..
                } => {
                    n_boxes_helper(function, variable, nestings, current_nestings)
                        && (*erased
                            || n_boxes_helper(argument, variable, nestings, current_nestings))
                }
                Put(term, _) => n_boxes_helper(term, variable, nestings, current_nestings + 1),
                Duplication {
                    expression, body, ..
                } => {
                    n_boxes_helper(expression, variable, nestings, current_nestings)
                        && n_boxes_helper(body, variable + 1, nestings, current_nestings)
                }

                Wrap(term, _) => n_boxes_helper(term, variable, nestings, current_nestings),
                Annotation { term, .. } => {
                    n_boxes_helper(term, variable, nestings, current_nestings)
                }
            }
        }

        n_boxes_helper(self, 0, nestings, 0)
    }

    fn is_recursive_in_helper<D: Definitions<T>>(
        &self,
        seen: &mut Vec<String>,
        definitions: &D,
    ) -> bool {
        use AnalysisTerm::*;

        match self {
            Variable(_, _) | Universe(_) | Hole(_) | Compressed(_) => false,
            Lambda { body, .. } => body.is_recursive_in_helper(seen, definitions),
            Application {
                function,
                argument,
                erased,
                ..
            } => {
                (!*erased && argument.is_recursive_in_helper(seen, definitions))
                    || function.is_recursive_in_helper(seen, definitions)
            }
            Put(term, _) => term.is_recursive_in_helper(seen, definitions),
            Duplication {
                expression, body, ..
            } => {
                expression.is_recursive_in_helper(seen, definitions)
                    || body.is_recursive_in_helper(seen, definitions)
            }
            Reference(reference, _) => {
                if seen.contains(reference) {
                    true
                } else {
                    if let Some(term) = definitions.get(reference) {
                        seen.push(reference.clone());
                        let res = term.as_ref().is_recursive_in_helper(seen, definitions);
                        seen.pop();
                        res
                    } else {
                        false
                    }
                }
            }
            Function {
                argument_type,
                return_type,
                ..
            } => {
                argument_type.is_recursive_in_helper(seen, definitions)
                    && return_type.is_recursive_in_helper(seen, definitions)
            }
            Annotation { term, .. } => term.is_recursive_in_helper(seen, definitions),
            Wrap(term, _) => term.is_recursive_in_helper(seen, definitions),
        }
    }

    pub fn is_recursive_in<D: Definitions<T>>(&self, definitions: &D) -> bool {
        self.is_recursive_in_helper(&mut vec![], definitions)
    }

    pub fn is_stratified(&self) -> Result<(), StratificationError> {
        use AnalysisTerm::*;

        match &self {
            Lambda { body, erased, .. } => {
                if body.uses() > if *erased { 0 } else { 1 } {
                    return Err(StratificationError::MultiplicityMismatch);
                }
                if !body.is_boxed_n_times(0) {
                    return Err(StratificationError::AffineUsedInBox);
                }

                body.is_stratified()?;
            }
            Application {
                function,
                argument,
                erased,
                ..
            } => {
                function.is_stratified()?;
                if !*erased {
                    argument.is_stratified()?;
                }
            }
            Put(term, _) => {
                term.is_stratified()?;
            }
            Duplication {
                body, expression, ..
            } => {
                if !body.is_boxed_n_times(1) {
                    return Err(StratificationError::DupNonUnitBoxMultiplicity);
                }
                expression.is_stratified()?;
                body.is_stratified()?;
            }
            Variable(_, _)
            | Reference(_, _)
            | Function { .. }
            | Universe(_)
            | Hole(_)
            | Compressed(_) => {}

            Wrap(term, _) => term.is_stratified()?,
            Annotation { term, .. } => {
                term.is_stratified()?;
            }
        }

        Ok(())
    }
}
