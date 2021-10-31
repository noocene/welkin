use welkin_core::term::{self, EqualityCache};

use super::{normalize::NormalizationError, AnalysisTerm, TypedDefinitions};

#[derive(Debug)]
pub enum AnalysisError<T> {
    NormalizationError(NormalizationError),
    CoreNormalizationError(term::NormalizationError),
    NonFunctionLambda {
        term: AnalysisTerm<T>,
        ty: AnalysisTerm<T>,
    },
    TypeError {
        expected: AnalysisTerm<T>,
        got: AnalysisTerm<T>,
    },
    ErasureMismatch {
        lambda: AnalysisTerm<T>,
        ty: AnalysisTerm<T>,
    },
    UnboundReference(String),
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

impl<T> From<NormalizationError> for AnalysisError<T> {
    fn from(e: NormalizationError) -> Self {
        AnalysisError::NormalizationError(e)
    }
}

impl<T> From<term::NormalizationError> for AnalysisError<T> {
    fn from(e: term::NormalizationError) -> Self {
        AnalysisError::CoreNormalizationError(e)
    }
}

impl<T> AnalysisTerm<Option<T>> {
    pub fn infer_in<U: TypedDefinitions<Option<T>>>(
        &self,
        definitions: &U,
        cache: &mut impl EqualityCache,
    ) -> Result<AnalysisTerm<Option<T>>, AnalysisError<Option<T>>>
    where
        T: Clone,
    {
        use AnalysisTerm::*;

        Ok(match self {
            Universe(_) => Universe(None),
            Annotation { ty, term, checked } => {
                if !checked {
                    term.check_in(ty, definitions, cache)?;
                }
                *ty.clone()
            }
            Reference(name, _) => {
                if let Some(dr) = definitions.get_typed(name) {
                    dr.as_ref().0.clone()
                } else {
                    Err(AnalysisError::UnboundReference(name.clone()))?
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
                argument_type.check_in(&Universe(None), definitions, &mut *cache)?;
                let mut return_type = return_type.clone();
                return_type.substitute_function_in(self_annotation, &argument_annotation);
                return_type.check_in(&Universe(None), definitions, cache)?;
                Universe(None)
            }
            Application {
                function,
                argument,
                erased,
                ..
            } => {
                let mut function_type = function.infer_in(definitions, &mut *cache)?;
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
                    argument.check_in(argument_type, definitions, cache)?;
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
                let expression_ty = expression.infer_in(definitions, cache)?;
                if let AnalysisTerm::Universe(_) = expression_ty {
                } else {
                    Err(AnalysisError::InvalidWrap {
                        got: expression_ty,
                        wrap: self.clone(),
                    })?;
                }
                Universe(None)
            }
            Put(expression, _) => Wrap(Box::new(expression.infer_in(definitions, cache)?), None),

            _ => Err(AnalysisError::Impossible(self.clone()))?,
        })
    }
}
