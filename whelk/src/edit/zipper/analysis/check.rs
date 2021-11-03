use std::fmt::Debug;
use welkin_core::term::EqualityCache;

use super::{infer::AnalysisError, AnalysisTerm, TypedDefinitions};

impl<T> AnalysisTerm<Option<T>> {
    pub fn check_in<
        U: TypedDefinitions<Option<T>>,
        F: FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
        G: FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
    >(
        &self,
        ty: &AnalysisTerm<Option<T>>,
        definitions: &U,
        annotate: &mut F,
        fill_hole: &mut G,
        cache: &mut impl EqualityCache,
    ) -> Result<(), AnalysisError<Option<T>>>
    where
        T: Clone + Debug,
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
                            annotation: self.annotation().cloned(),
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
                    body.check_in(
                        &*return_type,
                        definitions,
                        &mut *annotate,
                        &mut *fill_hole,
                        cache,
                    )?;
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
                let mut expression_ty = expression.infer_in(
                    definitions,
                    &mut *annotate,
                    &mut *fill_hole,
                    &mut *cache,
                )?;
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
                body.check_in(
                    &reduced,
                    definitions,
                    &mut *annotate,
                    &mut *fill_hole,
                    cache,
                )?;
            }
            Put(term, _) => {
                if let Wrap(ty, _) = reduced {
                    term.check_in(&ty, definitions, &mut *annotate, &mut *fill_hole, cache)?;
                } else {
                    Err(AnalysisError::ExpectedWrap {
                        term: self.clone(),
                        ty: reduced,
                    })?;
                }
            }
            term => {
                let inferred = if let AnalysisTerm::Hole(_) = self {
                    annotate(term.annotation(), ty);
                    ty.clone()
                } else {
                    let i =
                        self.infer_in(definitions, &mut *annotate, &mut *fill_hole, &mut *cache)?;
                    annotate(term.annotation(), &i);
                    i
                };

                if !inferred.equivalent_in(&reduced, definitions, &mut *fill_hole, cache)? {
                    Err(AnalysisError::TypeError {
                        expected: ty.clone(),
                        annotation: self.annotation().cloned(),
                        got: inferred,
                    })?;
                }
            }
        })
    }
}
