use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use welkin_core::term::EqualityCache;

use crate::edit::zipper::analysis::BasicContext;

use super::{normalize::NormalizationError, AnalysisTerm, TypedDefinitions};

#[derive(Debug, Serialize, Deserialize)]
pub enum AnalysisError<T> {
    NormalizationError(NormalizationError),
    NonFunctionLambda {
        term: AnalysisTerm<T>,
        ty: AnalysisTerm<T>,
    },
    TypeError {
        expected: AnalysisTerm<T>,
        got: AnalysisTerm<T>,
        annotation: T,
    },
    ErasureMismatch {
        lambda: AnalysisTerm<T>,
        ty: AnalysisTerm<T>,
        annotation: T,
    },
    UnboundReference {
        name: String,
        annotation: T,
    },
    NonFunctionApplication(AnalysisTerm<T>),
    UnboxedDuplication {
        term: AnalysisTerm<T>,
        ty: AnalysisTerm<T>,
    },
    Impossible(AnalysisTerm<T>),
    ExpectedWrap {
        term: AnalysisTerm<T>,
        ty: AnalysisTerm<T>,
    },
    InvalidWrap {
        wrap: AnalysisTerm<T>,
        got: AnalysisTerm<T>,
    },
}

impl<T> AnalysisError<T> {
    pub fn map_annotation<U, F: FnMut(T) -> U>(self, call: &mut F) -> AnalysisError<U> {
        match self {
            AnalysisError::NormalizationError(e) => AnalysisError::NormalizationError(e),
            AnalysisError::NonFunctionLambda { term, ty } => AnalysisError::NonFunctionLambda {
                term: term.map_annotation(&mut *call),
                ty: ty.map_annotation(&mut *call),
            },
            AnalysisError::TypeError {
                expected,
                got,
                annotation,
            } => AnalysisError::TypeError {
                expected: expected.map_annotation(&mut *call),
                got: got.map_annotation(&mut *call),
                annotation: call(annotation),
            },
            AnalysisError::ErasureMismatch {
                lambda,
                ty,
                annotation,
            } => AnalysisError::ErasureMismatch {
                lambda: lambda.map_annotation(&mut *call),
                ty: ty.map_annotation(&mut *call),
                annotation: call(annotation),
            },
            AnalysisError::UnboundReference { name, annotation } => {
                AnalysisError::UnboundReference {
                    name,
                    annotation: call(annotation),
                }
            }
            AnalysisError::NonFunctionApplication(ty) => {
                AnalysisError::NonFunctionApplication(ty.map_annotation(&mut *call))
            }
            AnalysisError::UnboxedDuplication { term, ty } => AnalysisError::UnboxedDuplication {
                term: term.map_annotation(&mut *call),
                ty: ty.map_annotation(&mut *call),
            },
            AnalysisError::Impossible(term) => {
                AnalysisError::Impossible(term.map_annotation(&mut *call))
            }
            AnalysisError::ExpectedWrap { term, ty } => AnalysisError::ExpectedWrap {
                term: term.map_annotation(&mut *call),
                ty: ty.map_annotation(&mut *call),
            },
            AnalysisError::InvalidWrap { wrap, got } => AnalysisError::InvalidWrap {
                wrap: wrap.map_annotation(&mut *call),
                got: got.map_annotation(&mut *call),
            },
        }
    }
}

impl<T> From<NormalizationError> for AnalysisError<T> {
    fn from(e: NormalizationError) -> Self {
        AnalysisError::NormalizationError(e)
    }
}

impl<T> AnalysisTerm<Option<T>> {
    pub fn infer_in<
        U: TypedDefinitions<Option<T>>,
        F: FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
        G: FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
    >(
        &self,
        definitions: &U,
        annotate: &mut F,
        fill_hole: &mut G,
        cache: &mut impl EqualityCache,
    ) -> Result<AnalysisTerm<Option<T>>, AnalysisError<Option<T>>>
    where
        T: Clone + Debug,
    {
        use AnalysisTerm::*;

        Ok(match self {
            Universe(_) => Universe(None),
            Annotation { ty, term, checked } => {
                if !checked {
                    term.check_in(ty, definitions, &mut *annotate, &mut *fill_hole, cache)?;
                }
                *ty.clone()
            }
            Reference(name, _) => {
                if let Some(dr) = definitions.get_typed(name) {
                    dr.as_ref().0.clone()
                } else {
                    Err(AnalysisError::UnboundReference {
                        name: name.clone(),
                        annotation: self.annotation().cloned(),
                    })?
                }
            }
            Function {
                argument_type,
                return_type,
                ..
            } => {
                let self_annotation = AnalysisTerm::Annotation {
                    checked: true,
                    term: Box::new(AnalysisTerm::Variable(1, None)),
                    ty: Box::new(self.clone()),
                };
                let argument_annotation = AnalysisTerm::Annotation {
                    checked: true,
                    term: Box::new(AnalysisTerm::Variable(0, None)),
                    ty: argument_type.clone(),
                };
                argument_type.check_in(
                    &Universe(None),
                    definitions,
                    &mut *annotate,
                    &mut *fill_hole,
                    &mut *cache,
                )?;
                let mut return_type = return_type.clone();
                return_type.substitute_function_in(self_annotation, &argument_annotation);
                return_type.check_in(
                    &Universe(None),
                    definitions,
                    &mut *annotate,
                    &mut *fill_hole,
                    cache,
                )?;
                Universe(None)
            }
            Application {
                function,
                argument,
                erased,
                ..
            } => {
                let mut function_type =
                    function.infer_in(definitions, &mut *annotate, &mut *fill_hole, &mut *cache)?;
                function_type.weak_normalize_in(definitions)?;
                if let Function {
                    argument_type,
                    return_type,
                    erased: function_erased,
                    ..
                } = &function_type
                {
                    if erased != function_erased {
                        Err(AnalysisError::ErasureMismatch {
                            lambda: self.clone(),
                            ty: function_type.clone(),
                            annotation: self.annotation().cloned(),
                        })?;
                    }
                    let self_annotation = AnalysisTerm::Annotation {
                        term: function.clone(),
                        ty: Box::new(function_type.clone()),
                        checked: true,
                    };
                    let argument_annotation = AnalysisTerm::Annotation {
                        term: argument.clone(),
                        ty: argument_type.clone(),
                        checked: true,
                    };
                    argument.check_in(
                        argument_type,
                        definitions,
                        &mut *annotate,
                        &mut *fill_hole,
                        cache,
                    )?;
                    let mut return_type = return_type.clone();
                    return_type.substitute_function_in(self_annotation, &argument_annotation);
                    return_type.weak_normalize_in(definitions)?;
                    *return_type
                } else {
                    Err(AnalysisError::NonFunctionApplication(*function.clone()))?
                }
            }
            Variable { .. } => self.clone(),

            Wrap(expression, _) => {
                let expression_ty =
                    expression.infer_in(definitions, &mut *annotate, &mut *fill_hole, cache)?;
                if let AnalysisTerm::Universe(_) = expression_ty {
                } else {
                    Err(AnalysisError::InvalidWrap {
                        got: expression_ty,
                        wrap: self.clone(),
                    })?;
                }
                Universe(None)
            }
            Put(expression, _) => Wrap(
                Box::new(expression.infer_in(
                    definitions,
                    &mut *annotate,
                    &mut *fill_hole,
                    cache,
                )?),
                None,
            ),
            Compressed(term) => {
                let ty = term.concrete_ty();

                if let Some(ty) = ty {
                    AnalysisTerm::from_unit_term_and_context(ty, &mut BasicContext::new())
                } else {
                    Err(AnalysisError::Impossible(self.clone()))?
                }
            }

            _ => Err(AnalysisError::Impossible(self.clone()))?,
        })
    }
}
