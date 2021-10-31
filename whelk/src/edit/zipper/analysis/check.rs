use welkin_core::term::{self, EqualityCache};

use crate::edit::zipper::analysis::DefAdapter;

use super::{infer::AnalysisError, AnalysisTerm, TypedDefinitions};

impl<T> AnalysisTerm<Option<T>> {
    pub fn check_in<
        U: TypedDefinitions<Option<T>>,
        F: FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
    >(
        &self,
        ty: &AnalysisTerm<Option<T>>,
        definitions: &U,
        annotate: &mut F,
        cache: &mut impl EqualityCache,
    ) -> Result<(), AnalysisError<Option<T>>>
    where
        T: Clone + std::fmt::Debug,
    {
        use AnalysisTerm::*;

        let mut reduced = ty.clone();
        reduced.weak_normalize_in(definitions)?;

        Ok(match self {
            Lambda { body, erased, .. } => {
                if let Function {
                    argument_type,
                    mut return_type,
                    erased: function_erased,
                    ..
                } = reduced
                {
                    if *erased != function_erased {
                        Err(AnalysisError::ErasureMismatch {
                            lambda: self.clone(),
                            ty: ty.clone(),
                        })?;
                    }
                    let self_annotation = Annotation {
                        checked: true,
                        term: Box::new(self.clone()),
                        ty: Box::new(ty.clone()),
                    };
                    let mut argument_annotation = AnalysisTerm::Annotation {
                        checked: true,
                        ty: argument_type,
                        term: Box::new(AnalysisTerm::Variable(0, None)),
                    };

                    return_type
                        .substitute_function_in_unshifted(self_annotation, &argument_annotation);

                    if let AnalysisTerm::Annotation { ty, .. } = &mut argument_annotation {
                        ty.shift_top();
                    } else {
                        panic!()
                    };

                    let mut body = body.clone();
                    body.substitute_top_in_unshifted(&argument_annotation);
                    body.check_in(&*return_type, definitions, &mut *annotate, cache)?;
                } else {
                    Err(AnalysisError::NonFunctionLambda {
                        term: self.clone(),
                        ty: ty.clone(),
                    })?
                }
            }
            Duplication {
                expression, body, ..
            } => {
                let mut expression_ty =
                    expression.infer_in(definitions, &mut *annotate, &mut *cache)?;
                expression_ty.weak_normalize_in(definitions)?;
                let expression_ty = if let Wrap(term, _) = expression_ty {
                    term
                } else {
                    Err(AnalysisError::UnboxedDuplication {
                        term: self.clone(),
                        ty: expression_ty.clone(),
                    })?
                };
                let argument_annotation = AnalysisTerm::Annotation {
                    checked: true,
                    ty: expression_ty,
                    term: Box::new(AnalysisTerm::Variable(0, None)),
                };
                let mut body = body.clone();
                body.substitute_top_in(&argument_annotation);
                body.check_in(&reduced, definitions, &mut *annotate, cache)?;
            }
            Put(term, _) => {
                if let Wrap(ty, _) = reduced {
                    term.check_in(&ty, definitions, &mut *annotate, cache)?;
                } else {
                    Err(AnalysisError::ExpectedWrap {
                        term: self.clone(),
                        ty: reduced,
                    })?;
                }
            }
            term => {
                annotate(term.annotation(), ty);
                let i = self.infer_in(definitions, &mut *annotate, &mut *cache)?;
                let inferred: term::Term<String> = i.clone().into();
                let reduced: term::Term<String> = reduced.into();
                if !inferred.equivalent(&reduced, &DefAdapter::new(&*definitions), cache)? {
                    Err(AnalysisError::TypeError {
                        expected: ty.clone(),
                        got: i,
                    })?;
                }
            }
        })
    }
}
