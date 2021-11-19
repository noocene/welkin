use std::mem::replace;

use serde::{Deserialize, Serialize};

use crate::edit::{
    dynamic::abst::controls::Zero,
    zipper::{Cursor, Path},
};

use std::fmt::Debug;

use super::{AnalysisTerm, Definitions};

#[derive(Debug, Serialize, Deserialize)]
pub enum NormalizationError {
    InvalidDuplication,
    InvalidApplication,
}

impl<T: Zero> AnalysisTerm<T> {
    pub fn normalize_in<U: Definitions<T>>(
        &mut self,
        definitions: &U,
    ) -> Result<(), NormalizationError>
    where
        T: Clone + Debug,
    {
        use AnalysisTerm::*;

        match self {
            Reference(binding, _) => {
                if let Some(term) = definitions.get(binding).map(|term| {
                    let mut term = term.as_ref().clone();
                    term.normalize_in(definitions)?;
                    Ok(term)
                }) {
                    *self = term?;
                }
            }
            Lambda { body, erased, .. } => {
                body.normalize_in(definitions)?;
                if *erased {
                    body.substitute_top_in(&AnalysisTerm::Variable(0, T::zero()));
                    *self = replace(&mut *body, Universe(T::zero()));
                }
            }
            Put(term, _) => {
                term.normalize_in(definitions)?;
                *self = replace(term, AnalysisTerm::Universe(T::zero()));
            }
            Duplication {
                body, expression, ..
            } => {
                body.substitute_top_in(expression);
                body.normalize_in(definitions)?;
                *self = replace(body, AnalysisTerm::Universe(T::zero()));
            }
            Application {
                function,
                argument,
                erased,
                ..
            } => {
                function.normalize_in(definitions)?;
                let function = *function.clone();
                if *erased {
                    *self = function;
                } else {
                    match function {
                        Put(_, _) => Err(NormalizationError::InvalidApplication)?,
                        Lambda { mut body, .. } => {
                            body.substitute_top_in(argument);
                            body.normalize_in(definitions)?;

                            *self = *body;
                        }
                        _ => {
                            argument.normalize_in(definitions)?;
                        }
                    }
                }
            }
            Variable(_, _)
            | Universe(_)
            | Wrap(_, _)
            | Function { .. }
            | Hole(_)
            | Compressed(_) => {}

            Annotation { term, .. } => {
                term.normalize_in(definitions)?;
                *self = replace(term, AnalysisTerm::Universe(T::zero()));
            }
        }

        Ok(())
    }
}

impl<T> AnalysisTerm<Option<T>> {
    pub(crate) fn decompress(&mut self) {
        match self {
            AnalysisTerm::Compressed(data) => {
                let data = Cursor::<()>::from_term_and_path(data.expand(), Path::Top);
                let data: AnalysisTerm<Option<()>> = data.into();
                *self = data.map_annotation(&mut |data| None);
            }
            _ => {}
        }
    }

    pub(crate) fn weak_normalize_in_erased<U: Definitions<Option<T>>>(
        &mut self,
        definitions: &U,
        erase: bool,
    ) -> Result<(), NormalizationError>
    where
        T: Clone,
    {
        use AnalysisTerm::*;

        match self {
            Reference(binding, _) => {
                if let Some(term) = definitions.get(binding).map(|term| {
                    let mut term = term.as_ref().clone();
                    term.weak_normalize_in_erased(definitions, erase)?;
                    Ok(term)
                }) {
                    *self = term?;
                }
            }
            Application {
                function,
                argument,
                erased,
                ..
            } => {
                function.weak_normalize_in_erased(definitions, erase)?;
                // argument.decompress();
                let f = *function.clone();
                match f {
                    Put(_, _) => Err(NormalizationError::InvalidApplication)?,
                    Duplication {
                        body,
                        expression,
                        annotation,
                        binder,
                    } => {
                        let mut argument = argument.clone();
                        argument.shift_top();
                        let body = Box::new(Application {
                            function: body,
                            argument,
                            erased: *erased,
                            annotation: None,
                        });
                        *self = Duplication {
                            expression,
                            body,
                            annotation,
                            binder,
                        };
                    }
                    Lambda { mut body, .. } => {
                        body.substitute_top_in(argument);
                        body.weak_normalize_in_erased(definitions, erase)?;
                        *self = *body;
                    }
                    _ => {}
                }
            }

            Put(term, _) if erase => {
                term.weak_normalize_in_erased(definitions, erase)?;
                *self = replace(term, AnalysisTerm::Universe(None));
            }

            Duplication {
                body, expression, ..
            } if erase => {
                body.substitute_top_in(expression);
                body.weak_normalize_in_erased(definitions, erase)?;
                *self = replace(body, AnalysisTerm::Universe(None));
            }

            Variable(_, _) | Lambda { .. } | Put(_, _) => {}

            Duplication {
                body, expression, ..
            } => {
                expression.weak_normalize_in_erased(definitions, erase)?;

                // TODO what is the correct annotation handling here
                match &mut **expression {
                    Put(term, _) => {
                        body.substitute_top_in(term);
                        body.weak_normalize_in_erased(definitions, erase)?;
                        *self = replace(body, AnalysisTerm::Universe(None));
                    }
                    Duplication {
                        body: sub_body,
                        expression: sub_expression,
                        ..
                    } => {
                        body.shift(1);
                        let dup = Duplication {
                            body: body.clone(),
                            expression: sub_body.clone(),
                            annotation: None,
                            binder: None,
                        };
                        *self = Duplication {
                            expression: Box::new(replace(
                                sub_expression,
                                AnalysisTerm::Universe(None),
                            )),
                            body: Box::new(dup),
                            annotation: None,
                            binder: None,
                        };
                    }
                    _ => {}
                }
            }

            Universe(_) | Function { .. } | Wrap(_, _) | Hole(_) | Compressed(_) => {}
            Annotation { term, .. } => {
                term.weak_normalize_in_erased(definitions, erase)?;
                *self = replace(term, AnalysisTerm::Universe(None));
            }
        }

        Ok(())
    }

    pub fn weak_normalize_in<U: Definitions<Option<T>>>(
        &mut self,
        definitions: &U,
    ) -> Result<(), NormalizationError>
    where
        T: Clone,
    {
        self.weak_normalize_in_erased(definitions, false)
    }
}
