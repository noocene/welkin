use bumpalo::Bump;
use std::fmt::Debug;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use welkin_core::term::EqualityCache;
type BumpBox<'a, T> = bumpalo::boxed::Box<'a, T>;

use super::{normalize::NormalizationError, AnalysisTerm, Definitions, TypedDefinitions};

enum EqualityTree<'a, T> {
    Equal(AnalysisTerm<T>, AnalysisTerm<T>),
    Or(BumpBox<'a, Option<(EqualityTree<'a, T>, EqualityTree<'a, T>)>>),
    And(BumpBox<'a, Option<(EqualityTree<'a, T>, EqualityTree<'a, T>)>>),
    Leaf(bool),
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
            tree: EqualityTree<'b, Option<T>>,
            definitions: &U,
            o_alloc: &'b Bump,
            fill_hole: &mut impl FnMut(Option<&T>, &AnalysisTerm<Option<T>>),
            cache: &mut impl EqualityCache,
        ) -> Result<EqualityTree<'b, Option<T>>, NormalizationError> {
            Ok(match tree {
                this @ EqualityTree::Leaf(_) => this,
                EqualityTree::And(mut data) => {
                    let (a, b) = data.as_ref().as_ref().unwrap();
                    match (a, b) {
                        (EqualityTree::Leaf(false), _) | (_, EqualityTree::Leaf(false)) => {
                            EqualityTree::Leaf(false)
                        }
                        (EqualityTree::Leaf(true), EqualityTree::Leaf(true)) => {
                            EqualityTree::Leaf(true)
                        }
                        _ => EqualityTree::And(BumpBox::new_in(
                            Some({
                                let data = data.take().unwrap();
                                (
                                    equivalence_helper(
                                        data.0,
                                        definitions,
                                        o_alloc,
                                        fill_hole,
                                        &mut *cache,
                                    )?,
                                    equivalence_helper(
                                        data.1,
                                        definitions,
                                        o_alloc,
                                        fill_hole,
                                        cache,
                                    )?,
                                )
                            }),
                            o_alloc,
                        )),
                    }
                }
                EqualityTree::Or(mut data) => {
                    let (a, b) = data.as_ref().as_ref().unwrap();
                    match (&a, &b) {
                        (EqualityTree::Leaf(true), _) | (_, EqualityTree::Leaf(true)) => {
                            EqualityTree::Leaf(true)
                        }
                        (EqualityTree::Leaf(false), EqualityTree::Leaf(false)) => {
                            EqualityTree::Leaf(false)
                        }
                        _ => EqualityTree::Or(BumpBox::new_in(
                            {
                                let data = data.take().unwrap();
                                Some((
                                    equivalence_helper(
                                        data.0,
                                        definitions,
                                        o_alloc,
                                        fill_hole,
                                        &mut *cache,
                                    )?,
                                    equivalence_helper(
                                        data.1,
                                        definitions,
                                        o_alloc,
                                        fill_hole,
                                        cache,
                                    )?,
                                ))
                            },
                            o_alloc,
                        )),
                    }
                }
                EqualityTree::Equal(mut a, mut b) => {
                    a.weak_normalize_in_erased(&Empty, true)?;
                    b.weak_normalize_in_erased(&Empty, true)?;

                    let mut hasher = DefaultHasher::new();

                    a.hash(&mut hasher);

                    let a_hash = hasher.finish();

                    let mut hasher = DefaultHasher::new();

                    b.hash(&mut hasher);

                    let b_hash = hasher.finish();

                    if a_hash == b_hash {
                        return Ok(EqualityTree::Leaf(true));
                    }

                    if let Some(leaf) = cache.check(a_hash, b_hash) {
                        return Ok(EqualityTree::Leaf(leaf));
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
                                        EqualityTree::Equal(*a_argument_type, *b_argument_type),
                                        EqualityTree::Equal(*a_return_type, *b_return_type),
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
                                EqualityTree::Equal(*a_body, *b_body)
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
                                        EqualityTree::Equal(*a_argument, *b_argument),
                                        EqualityTree::Equal(*a_function, *b_function),
                                    )),
                                    o_alloc,
                                ))
                            }
                        }
                        (Variable(a, _), Variable(b, _)) => EqualityTree::Leaf(a == b),
                        (Wrap(a, _), Wrap(b, _)) => EqualityTree::Equal(*a, *b),
                        (Put(a, _), Put(b, _)) => EqualityTree::Equal(*a, *b),
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
                                EqualityTree::Equal(*a_expression, *b_expression),
                                EqualityTree::Equal(*a_body, *b_body),
                            )),
                            o_alloc,
                        )),
                        _ => EqualityTree::Leaf(false),
                    };

                    if let Some(ret_a) = ret_a {
                        EqualityTree::Or(BumpBox::new_in(Some((ret_a, ret_b)), o_alloc))
                    } else {
                        ret_b
                    }
                }
            })
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
            equality = equivalence_helper(equality, definitions, &o_alloc, fill_hole, cache)?;
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
