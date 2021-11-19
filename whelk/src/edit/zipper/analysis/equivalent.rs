use bumpalo::Bump;
use std::fmt::{self, Debug};
use std::mem::replace;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use welkin_core::term::EqualityCache;
type BumpBox<'a, T> = bumpalo::boxed::Box<'a, T>;

use crate::edit::zipper::analysis::BasicContext;

use super::{normalize::NormalizationError, AnalysisTerm, Definitions, TypedDefinitions};

enum EqualityTree<'a, T> {
    Equal(AnalysisTerm<T>, AnalysisTerm<T>),
    Or(BumpBox<'a, Option<(EqualityTree<'a, T>, EqualityTree<'a, T>)>>),
    And(BumpBox<'a, Option<(EqualityTree<'a, T>, EqualityTree<'a, T>)>>),
    Leaf(bool),
}

impl<'a, T: Debug> Debug for EqualityTree<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal(arg0, arg1) => f.debug_tuple("Equal").field(arg0).field(arg1).finish(),
            Self::Or(arg0) => f.debug_tuple("Or").field(arg0).finish(),
            Self::And(arg0) => f.debug_tuple("And").field(arg0).finish(),
            Self::Leaf(arg0) => f.debug_tuple("Leaf").field(arg0).finish(),
        }
    }
}

struct Empty;

impl<T> TypedDefinitions<T> for Empty {
    fn get_typed(
        &self,
        name: &str,
    ) -> Option<super::DefinitionResult<(AnalysisTerm<T>, AnalysisTerm<T>)>> {
        None
    }
}

impl<T> AnalysisTerm<Option<T>> {
    pub fn equivalent_in<U: Definitions<Option<T>>>(
        &self,
        other: &Self,
        definitions: &U,
        fill_hole: &mut impl FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
        cache: &mut impl EqualityCache,
    ) -> Result<bool, NormalizationError>
    where
        T: Clone + Debug,
    {
        use AnalysisTerm::*;

        fn equivalence_helper<'b, U: Definitions<Option<T>>, T: Clone + Debug>(
            tree: &mut EqualityTree<'b, Option<T>>,
            definitions: &U,
            o_alloc: &'b Bump,
            fill_hole: &mut impl FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
            cache: &mut impl EqualityCache,
        ) -> Result<(), NormalizationError> {
            *tree = Ok(match tree {
                EqualityTree::Leaf(data) => EqualityTree::Leaf(*data),
                EqualityTree::And(ref mut data) => {
                    let (a, b) = data.as_ref().as_ref().unwrap();
                    match (a, b) {
                        (EqualityTree::Leaf(false), _) | (_, EqualityTree::Leaf(false)) => {
                            EqualityTree::Leaf(false)
                        }
                        (EqualityTree::Leaf(true), EqualityTree::Leaf(true)) => {
                            EqualityTree::Leaf(true)
                        }
                        (EqualityTree::Leaf(true), a) => data.take().unwrap().1,
                        (a, EqualityTree::Leaf(true)) => data.take().unwrap().0,
                        _ => {
                            let mut data = data.take().unwrap();
                            equivalence_helper(
                                &mut data.0,
                                definitions,
                                o_alloc,
                                fill_hole,
                                &mut *cache,
                            )?;
                            equivalence_helper(
                                &mut data.1,
                                definitions,
                                o_alloc,
                                fill_hole,
                                cache,
                            )?;
                            EqualityTree::And(BumpBox::new_in(Some((data.0, data.1)), o_alloc))
                        }
                    }
                }
                EqualityTree::Or(ref mut data) => {
                    let (a, b) = data.as_ref().as_ref().unwrap();
                    match (&a, &b) {
                        (EqualityTree::Leaf(true), _) | (_, EqualityTree::Leaf(true)) => {
                            EqualityTree::Leaf(true)
                        }
                        (EqualityTree::Leaf(false), EqualityTree::Leaf(false)) => {
                            EqualityTree::Leaf(false)
                        }
                        (EqualityTree::Leaf(false), a) => data.take().unwrap().1,
                        (a, EqualityTree::Leaf(true)) => data.take().unwrap().0,
                        _ => EqualityTree::Or(BumpBox::new_in(
                            {
                                let mut data = data.take().unwrap();
                                equivalence_helper(
                                    &mut data.0,
                                    definitions,
                                    o_alloc,
                                    fill_hole,
                                    &mut *cache,
                                )?;
                                equivalence_helper(
                                    &mut data.1,
                                    definitions,
                                    o_alloc,
                                    fill_hole,
                                    cache,
                                )?;
                                Some((data.0, data.1))
                            },
                            o_alloc,
                        )),
                    }
                }
                EqualityTree::Equal(ref mut a, ref mut b) => {
                    a.weak_normalize_in_erased(&Empty, true)?;
                    b.weak_normalize_in_erased(&Empty, true)?;

                    let mut hasher = DefaultHasher::new();

                    a.hash(&mut hasher);

                    let a_hash = hasher.finish();

                    let mut hasher = DefaultHasher::new();

                    b.hash(&mut hasher);

                    let b_hash = hasher.finish();

                    if a_hash == b_hash {
                        *tree = EqualityTree::Leaf(true);
                        return Ok(());
                    }

                    if let Some(leaf) = cache.check(a_hash, b_hash) {
                        *tree = EqualityTree::Leaf(leaf);
                        return Ok(());
                    }

                    let mut ret_a = None;

                    match (&a, &b) {
                        (
                            Application {
                                function: a_function,
                                argument: a_argument,
                                erased: a_erased,
                                ..
                            },
                            Application {
                                function: b_function,
                                argument: b_argument,
                                erased: b_erased,
                                ..
                            },
                        ) => {
                            ret_a = if a_erased != b_erased {
                                Some(EqualityTree::Leaf(false))
                            } else {
                                Some(EqualityTree::And(BumpBox::new_in(
                                    Some((
                                        EqualityTree::Equal(
                                            *a_function.clone(),
                                            *b_function.clone(),
                                        ),
                                        EqualityTree::Equal(
                                            *a_argument.clone(),
                                            *b_argument.clone(),
                                        ),
                                    )),
                                    o_alloc,
                                )))
                            }
                        }
                        (Reference(a, _), Reference(b, _)) => {
                            if a == b {
                                ret_a = Some(EqualityTree::Leaf(true))
                            }
                        }
                        (Hole(annotation), a) | (a, Hole(annotation)) => {
                            fill_hole(annotation.as_ref().clone(), a);
                            ret_a = Some(EqualityTree::Leaf(true));
                        }
                        (Compressed(a), Compressed(b)) => {
                            if let Some(eq) = a.partial_eq(b.as_ref()) {
                                *tree = EqualityTree::Leaf(eq);
                                return Ok(());
                            }
                        }
                        _ => {}
                    }

                    a.weak_normalize_in_erased(definitions, true)?;
                    b.weak_normalize_in_erased(definitions, true)?;

                    let ret_b = match (a, b) {
                        (Universe(_), Universe(_)) => EqualityTree::Leaf(true),
                        (
                            Function {
                                argument_type: a_argument_type,
                                return_type: a_return_type,
                                erased: a_erased,
                                ..
                            },
                            Function {
                                argument_type: b_argument_type,
                                return_type: b_return_type,
                                erased: b_erased,
                                ..
                            },
                        ) => {
                            if a_erased != b_erased {
                                EqualityTree::Leaf(false)
                            } else {
                                EqualityTree::And(BumpBox::new_in(
                                    Some((
                                        EqualityTree::Equal(
                                            replace(
                                                a_argument_type.as_mut(),
                                                AnalysisTerm::Hole(None),
                                            ),
                                            replace(
                                                b_argument_type.as_mut(),
                                                AnalysisTerm::Hole(None),
                                            ),
                                        ),
                                        EqualityTree::Equal(
                                            replace(
                                                a_return_type.as_mut(),
                                                AnalysisTerm::Hole(None),
                                            ),
                                            replace(
                                                b_return_type.as_mut(),
                                                AnalysisTerm::Hole(None),
                                            ),
                                        ),
                                    )),
                                    o_alloc,
                                ))
                            }
                        }
                        (
                            Lambda {
                                body: a_body,
                                erased: a_erased,
                                ..
                            },
                            Lambda {
                                body: b_body,
                                erased: b_erased,
                                ..
                            },
                        ) => {
                            if a_erased != b_erased {
                                EqualityTree::Leaf(false)
                            } else {
                                EqualityTree::Equal(
                                    replace(a_body.as_mut(), AnalysisTerm::Hole(None)),
                                    replace(b_body.as_mut(), AnalysisTerm::Hole(None)),
                                )
                            }
                        }
                        (
                            Application {
                                argument: a_argument,
                                function: a_function,
                                erased: a_erased,
                                ..
                            },
                            Application {
                                argument: b_argument,
                                function: b_function,
                                erased: b_erased,
                                ..
                            },
                        ) => {
                            if a_erased != b_erased {
                                EqualityTree::Leaf(false)
                            } else {
                                EqualityTree::And(BumpBox::new_in(
                                    Some((
                                        EqualityTree::Equal(
                                            replace(a_argument.as_mut(), AnalysisTerm::Hole(None)),
                                            replace(b_argument.as_mut(), AnalysisTerm::Hole(None)),
                                        ),
                                        EqualityTree::Equal(
                                            replace(a_function.as_mut(), AnalysisTerm::Hole(None)),
                                            replace(b_function.as_mut(), AnalysisTerm::Hole(None)),
                                        ),
                                    )),
                                    o_alloc,
                                ))
                            }
                        }
                        (Variable(a, _), Variable(b, _)) => EqualityTree::Leaf(a == b),
                        (Wrap(a, _), Wrap(b, _)) => EqualityTree::Equal(
                            replace(a.as_mut(), AnalysisTerm::Hole(None)),
                            replace(b.as_mut(), AnalysisTerm::Hole(None)),
                        ),
                        (Put(a, _), Put(b, _)) => EqualityTree::Equal(
                            replace(a.as_mut(), AnalysisTerm::Hole(None)),
                            replace(b.as_mut(), AnalysisTerm::Hole(None)),
                        ),
                        (
                            Duplication {
                                expression: a_expression,
                                body: a_body,
                                ..
                            },
                            Duplication {
                                expression: b_expression,
                                body: b_body,
                                ..
                            },
                        ) => EqualityTree::And(BumpBox::new_in(
                            Some((
                                EqualityTree::Equal(
                                    replace(a_expression.as_mut(), AnalysisTerm::Hole(None)),
                                    replace(b_expression.as_mut(), AnalysisTerm::Hole(None)),
                                ),
                                EqualityTree::Equal(
                                    replace(a_body.as_mut(), AnalysisTerm::Hole(None)),
                                    replace(b_body.as_mut(), AnalysisTerm::Hole(None)),
                                ),
                            )),
                            o_alloc,
                        )),
                        (Compressed(a), Compressed(b)) => {
                            if let Some(eq) = a.partial_eq(b.as_ref()) {
                                EqualityTree::Leaf(eq)
                            } else {
                                let a = AnalysisTerm::from_unit_term_and_context(
                                    a.expand(),
                                    &mut BasicContext::new(),
                                );
                                let b = AnalysisTerm::from_unit_term_and_context(
                                    b.expand(),
                                    &mut BasicContext::new(),
                                );
                                EqualityTree::Equal(a, b)
                            }
                        }
                        (Compressed(a), b) | (b, Compressed(a)) => {
                            let a = AnalysisTerm::from_unit_term_and_context(
                                a.expand(),
                                &mut BasicContext::new(),
                            );
                            EqualityTree::Equal(a, replace(b, AnalysisTerm::Hole(None)))
                        }
                        _ => EqualityTree::Leaf(false),
                    };

                    if let Some(ret_a) = ret_a {
                        EqualityTree::Or(BumpBox::new_in(Some((ret_a, ret_b)), o_alloc))
                    } else {
                        ret_b
                    }
                }
            })?;
            Ok(())
        }

        let o_alloc = Bump::new();

        let mut a = self.clone();
        let mut b = other.clone();

        let complete = a.is_complete() && b.is_complete();

        a.weak_normalize_in_erased(definitions, true)?;
        b.weak_normalize_in_erased(definitions, true)?;

        let mut hasher = DefaultHasher::new();

        a.hash(&mut hasher);

        let a_hash = hasher.finish();

        let mut hasher = DefaultHasher::new();

        b.hash(&mut hasher);

        let b_hash = hasher.finish();

        if a_hash == b_hash {
            return Ok(true);
        }

        if let Some(leaf) = cache.check(a_hash, b_hash) {
            return Ok(leaf);
        }

        let mut equality = EqualityTree::Equal(a, b);

        while match equality {
            EqualityTree::Leaf(_) => false,
            _ => true,
        } {
            equivalence_helper(&mut equality, definitions, &o_alloc, fill_hole, cache)?;
        }

        Ok(if let EqualityTree::Leaf(leaf) = equality {
            if complete {
                cache.register(a_hash, b_hash, leaf);
            }
            leaf
        } else {
            panic!()
        })
    }
}
