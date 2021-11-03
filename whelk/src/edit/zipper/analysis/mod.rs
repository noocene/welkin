mod check;
mod infer;
pub use infer::AnalysisError;
mod cursor;
mod equivalent;
mod is_complete;
mod normalize;
mod shift;
mod substitute;

use std::marker::PhantomData;

use derivative::Derivative;
use welkin_core::term::{self, Index};

use super::{Cursor, Term};

pub enum DefinitionResult<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> DefinitionResult<'a, T> {
    pub fn as_ref<'b>(&'b self) -> &'b T {
        match self {
            DefinitionResult::Borrowed(a) => a,
            DefinitionResult::Owned(a) => a,
        }
    }
}

pub trait Definitions<T> {
    fn get(&self, name: &str) -> Option<DefinitionResult<AnalysisTerm<T>>>;
}

pub struct DefAdapter<'a, T, U>(&'a T, PhantomData<U>);

impl<'a, T, U> DefAdapter<'a, T, U> {
    pub fn new(definitions: &'a T) -> Self {
        DefAdapter(definitions, PhantomData)
    }
}

pub trait TypedDefinitions<T> {
    fn get_typed(&self, name: &str)
        -> Option<DefinitionResult<(AnalysisTerm<T>, AnalysisTerm<T>)>>;
}

impl<T, U: TypedDefinitions<T>> Definitions<T> for U {
    fn get(&self, name: &str) -> Option<DefinitionResult<AnalysisTerm<T>>> {
        self.get_typed(name).map(|res| match res {
            DefinitionResult::Borrowed((_, term)) => DefinitionResult::Borrowed(term),
            DefinitionResult::Owned((_, term)) => DefinitionResult::Owned(term),
        })
    }
}

impl<'a, U: Clone, T: TypedDefinitions<U>> term::TypedDefinitions<String> for DefAdapter<'a, T, U> {
    fn get_typed(
        &self,
        name: &String,
    ) -> Option<term::DefinitionResult<(term::Term<String>, term::Term<String>)>> {
        TypedDefinitions::get_typed(self.0, name).map(|a| match a {
            DefinitionResult::Borrowed((ty, term)) => {
                term::DefinitionResult::Owned((ty.clone().into(), term.clone().into()))
            }
            DefinitionResult::Owned((ty, term)) => {
                term::DefinitionResult::Owned((ty.into(), term.into()))
            }
        })
    }
}

#[derive(Derivative, Debug, Clone)]
#[derivative(Hash(bound = ""))]
pub enum AnalysisTerm<T> {
    Lambda {
        erased: bool,
        name: Option<String>,
        body: Box<AnalysisTerm<T>>,
        #[derivative(Hash = "ignore")]
        annotation: T,
    },
    Variable(usize, #[derivative(Hash = "ignore")] T),
    Application {
        erased: bool,
        function: Box<AnalysisTerm<T>>,
        argument: Box<AnalysisTerm<T>>,
        #[derivative(Hash = "ignore")]
        annotation: T,
    },
    Put(Box<AnalysisTerm<T>>, #[derivative(Hash = "ignore")] T),
    Duplication {
        binder: Option<String>,
        expression: Box<AnalysisTerm<T>>,
        body: Box<AnalysisTerm<T>>,
        #[derivative(Hash = "ignore")]
        annotation: T,
    },
    Reference(String, #[derivative(Hash = "ignore")] T),

    Universe(#[derivative(Hash = "ignore")] T),
    Function {
        erased: bool,
        name: Option<String>,
        self_name: Option<String>,
        argument_type: Box<AnalysisTerm<T>>,
        return_type: Box<AnalysisTerm<T>>,
        #[derivative(Hash = "ignore")]
        annotation: T,
    },
    Wrap(Box<AnalysisTerm<T>>, #[derivative(Hash = "ignore")] T),

    Hole(#[derivative(Hash = "ignore")] T),

    Annotation {
        checked: bool,
        term: Box<AnalysisTerm<T>>,
        ty: Box<AnalysisTerm<T>>,
    },
}

impl<T> From<AnalysisTerm<T>> for Term<T> {
    fn from(term: AnalysisTerm<T>) -> Self {
        match term {
            AnalysisTerm::Lambda {
                erased,
                name,
                body,
                annotation,
            } => Term::Lambda {
                erased,
                name,
                body: Box::new((*body).into()),
                annotation,
            },
            AnalysisTerm::Variable(_, _) => todo!(),
            AnalysisTerm::Application {
                erased,
                function,
                argument,
                annotation,
            } => Term::Application {
                erased,
                function: Box::new((*function).into()),
                argument: Box::new((*argument).into()),
                annotation,
            },
            AnalysisTerm::Put(term, annotation) => Term::Put(Box::new((*term).into()), annotation),
            AnalysisTerm::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => Term::Duplication {
                binder,
                expression: Box::new((*expression).into()),
                body: Box::new((*body).into()),
                annotation,
            },
            AnalysisTerm::Reference(r, annotation) => Term::Reference(r, annotation),
            AnalysisTerm::Universe(annotation) => Term::Universe(annotation),
            AnalysisTerm::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => Term::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new((*argument_type).into()),
                return_type: Box::new((*return_type).into()),
                annotation,
            },
            AnalysisTerm::Wrap(term, annotation) => {
                Term::Wrap(Box::new((*term).into()), annotation)
            }
            AnalysisTerm::Hole(annotation) => Term::Hole(annotation),
            AnalysisTerm::Annotation { checked, term, ty } => (*term).into(),
        }
    }
}

impl<T> AnalysisTerm<Option<T>> {
    fn annotation(&self) -> Option<&T> {
        match self {
            AnalysisTerm::Lambda { annotation, .. } => annotation.as_ref().clone(),
            AnalysisTerm::Variable(_, annotation) => annotation.as_ref().clone(),
            AnalysisTerm::Application { annotation, .. } => annotation.as_ref().clone(),
            AnalysisTerm::Put(_, annotation) => annotation.as_ref().clone(),
            AnalysisTerm::Duplication { annotation, .. } => annotation.as_ref().clone(),
            AnalysisTerm::Reference(_, annotation) => annotation.as_ref().clone(),
            AnalysisTerm::Universe(annotation) => annotation.as_ref().clone(),
            AnalysisTerm::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => annotation.as_ref().clone(),
            AnalysisTerm::Wrap(_, annotation) => annotation.as_ref().clone(),
            AnalysisTerm::Hole(annotation) => annotation.as_ref().clone(),
            AnalysisTerm::Annotation { .. } => None,
        }
    }

    pub fn clear_annotation(self) -> AnalysisTerm<()> {
        match self {
            AnalysisTerm::Lambda {
                erased, name, body, ..
            } => AnalysisTerm::Lambda {
                erased,
                name,
                annotation: (),
                body: Box::new(body.clear_annotation()),
            },
            AnalysisTerm::Variable(idx, _) => AnalysisTerm::Variable(idx, ()),
            AnalysisTerm::Application {
                erased,
                function,
                argument,
                annotation,
            } => AnalysisTerm::Application {
                erased,
                function: Box::new(function.clear_annotation()),
                argument: Box::new(argument.clear_annotation()),
                annotation: (),
            },
            AnalysisTerm::Put(term, _) => AnalysisTerm::Put(Box::new(term.clear_annotation()), ()),
            AnalysisTerm::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => AnalysisTerm::Duplication {
                binder,
                expression: Box::new(expression.clear_annotation()),
                body: Box::new(body.clear_annotation()),
                annotation: (),
            },
            AnalysisTerm::Reference(r, _) => AnalysisTerm::Reference(r, ()),
            AnalysisTerm::Universe(_) => AnalysisTerm::Universe(()),
            AnalysisTerm::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => AnalysisTerm::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new(argument_type.clear_annotation()),
                return_type: Box::new(return_type.clear_annotation()),
                annotation: (),
            },
            AnalysisTerm::Wrap(term, _) => {
                AnalysisTerm::Wrap(Box::new(term.clear_annotation()), ())
            }
            AnalysisTerm::Hole(_) => AnalysisTerm::Hole(()),
            AnalysisTerm::Annotation { checked, term, ty } => AnalysisTerm::Annotation {
                checked,
                term: Box::new(term.clear_annotation()),
                ty: Box::new(ty.clear_annotation()),
            },
        }
    }
}

impl<T> AnalysisTerm<T> {
    fn map_annotation<U, F: Fn(T) -> U>(self, call: &F) -> AnalysisTerm<U> {
        match self {
            AnalysisTerm::Lambda {
                erased,
                name,
                body,
                annotation,
            } => AnalysisTerm::Lambda {
                erased,
                name,
                annotation: call(annotation),
                body: Box::new(body.map_annotation(&*call)),
            },
            AnalysisTerm::Variable(idx, annotation) => {
                AnalysisTerm::Variable(idx, call(annotation))
            }
            AnalysisTerm::Application {
                erased,
                function,
                argument,
                annotation,
            } => AnalysisTerm::Application {
                erased,
                function: Box::new(function.map_annotation(&*call)),
                argument: Box::new(argument.map_annotation(&*call)),
                annotation: call(annotation),
            },
            AnalysisTerm::Put(term, annotation) => {
                AnalysisTerm::Put(Box::new(term.map_annotation(&*call)), call(annotation))
            }
            AnalysisTerm::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => AnalysisTerm::Duplication {
                binder,
                expression: Box::new(expression.map_annotation(&*call)),
                body: Box::new(body.map_annotation(&*call)),
                annotation: call(annotation),
            },
            AnalysisTerm::Reference(r, annotation) => AnalysisTerm::Reference(r, call(annotation)),
            AnalysisTerm::Universe(annotation) => AnalysisTerm::Universe(call(annotation)),
            AnalysisTerm::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => AnalysisTerm::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new(argument_type.map_annotation(&*call)),
                return_type: Box::new(return_type.map_annotation(&*call)),
                annotation: call(annotation),
            },
            AnalysisTerm::Wrap(term, annotation) => {
                AnalysisTerm::Wrap(Box::new(term.map_annotation(&*call)), call(annotation))
            }
            AnalysisTerm::Hole(annotation) => AnalysisTerm::Hole(call(annotation)),
            AnalysisTerm::Annotation { checked, term, ty } => AnalysisTerm::Annotation {
                checked,
                term: Box::new(term.map_annotation(&*call)),
                ty: Box::new(ty.map_annotation(&*call)),
            },
        }
    }
}

impl<T: Clone> From<Cursor<T>> for AnalysisTerm<Option<T>> {
    fn from(data: Cursor<T>) -> Self {
        match data {
            Cursor::Lambda(cursor) => AnalysisTerm::Lambda {
                erased: cursor.erased(),
                name: cursor.name.clone(),
                annotation: Some(cursor.annotation.clone()),
                body: Box::new(cursor.body().into()),
            },
            Cursor::Application(cursor) => AnalysisTerm::Application {
                erased: cursor.erased(),
                annotation: Some(cursor.annotation.clone()),
                argument: Box::new(cursor.clone().argument().into()),
                function: Box::new(cursor.clone().function().into()),
            },
            Cursor::Put(cursor) => {
                let annotation = cursor.annotation.clone();
                AnalysisTerm::Put(Box::new(cursor.term().into()), Some(annotation))
            }
            Cursor::Reference(ref cursor) => {
                if let Some(idx) = data.context().position(|name| {
                    if let Some(name) = name {
                        if cursor.name() == &name {
                            return true;
                        }
                    }
                    false
                }) {
                    AnalysisTerm::Variable(idx, Some(cursor.annotation.clone()))
                } else {
                    AnalysisTerm::Reference(
                        cursor.name().to_owned(),
                        Some(cursor.annotation.clone()),
                    )
                }
            }
            Cursor::Duplication(cursor) => AnalysisTerm::Duplication {
                binder: cursor.binder.clone(),
                annotation: Some(cursor.annotation.clone()),
                expression: Box::new(cursor.clone().expression().into()),
                body: Box::new(cursor.clone().body().into()),
            },
            Cursor::Universe(cursor) => AnalysisTerm::Universe(Some(cursor.annotation)),
            Cursor::Function(cursor) => AnalysisTerm::Function {
                erased: cursor.erased(),
                annotation: Some(cursor.annotation.clone()),
                self_name: cursor.self_binder.clone(),
                name: cursor.binder.clone(),
                argument_type: Box::new(cursor.clone().argument_type().into()),
                return_type: Box::new(cursor.return_type().into()),
            },
            Cursor::Wrap(cursor) => {
                let annotation = cursor.annotation.clone();
                AnalysisTerm::Wrap(Box::new(cursor.term().into()), Some(annotation))
            }
            Cursor::Hole(cursor) => AnalysisTerm::Hole(Some(cursor.annotation)),

            Cursor::Dynamic(cursor) => {
                // TODO proper expansion here
                AnalysisTerm::Hole(None)
            }
        }
    }
}

impl<T> From<AnalysisTerm<T>> for term::Term<String> {
    fn from(term: AnalysisTerm<T>) -> Self {
        match term {
            AnalysisTerm::Lambda {
                erased,
                name,
                body,
                annotation,
            } => term::Term::Lambda {
                erased,
                body: Box::new((*body).into()),
            },

            AnalysisTerm::Variable(idx, _) => term::Term::Variable(Index(idx)),

            AnalysisTerm::Application {
                erased,
                function,
                argument,
                annotation,
            } => term::Term::Apply {
                erased,
                function: Box::new((*function).into()),
                argument: Box::new((*argument).into()),
            },

            AnalysisTerm::Put(term, _) => term::Term::Put(Box::new((*term).into())),

            AnalysisTerm::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => term::Term::Duplicate {
                expression: Box::new((*expression).into()),
                body: Box::new((*body).into()),
            },

            AnalysisTerm::Reference(r, _) => term::Term::Reference(r),

            AnalysisTerm::Universe(_) => term::Term::Universe,

            AnalysisTerm::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => term::Term::Function {
                erased,
                argument_type: Box::new((*argument_type).into()),
                return_type: Box::new((*return_type).into()),
            },

            AnalysisTerm::Wrap(term, _) => term::Term::Wrap(Box::new((*term).into())),

            AnalysisTerm::Hole(_) => todo!(),

            AnalysisTerm::Annotation { checked, term, ty } => term::Term::Annotation {
                checked,
                expression: Box::new((*term).into()),
                ty: Box::new((*ty).into()),
            },
        }
    }
}

impl<T> From<term::Term<String>> for AnalysisTerm<Option<T>> {
    fn from(term: term::Term<String>) -> Self {
        match term {
            term::Term::Lambda { erased, body } => AnalysisTerm::Lambda {
                erased,
                body: Box::new((*body).into()),
                annotation: None,
                name: None,
            },

            term::Term::Variable(Index(idx)) => AnalysisTerm::Variable(idx, None),

            term::Term::Apply {
                erased,
                function,
                argument,
            } => AnalysisTerm::Application {
                erased,
                function: Box::new((*function).into()),
                argument: Box::new((*argument).into()),
                annotation: None,
            },

            term::Term::Put(term) => AnalysisTerm::Put(Box::new((*term).into()), None),

            term::Term::Duplicate { expression, body } => AnalysisTerm::Duplication {
                expression: Box::new((*expression).into()),
                body: Box::new((*body).into()),
                annotation: None,
                binder: None,
            },

            term::Term::Reference(r) => AnalysisTerm::Reference(r, None),

            term::Term::Universe => AnalysisTerm::Universe(None),

            term::Term::Function {
                erased,
                argument_type,
                return_type,
            } => AnalysisTerm::Function {
                erased,
                argument_type: Box::new((*argument_type).into()),
                return_type: Box::new((*return_type).into()),
                annotation: None,
                self_name: None,
                name: None,
            },

            term::Term::Wrap(term) => AnalysisTerm::Wrap(Box::new((*term).into()), None),

            term::Term::Annotation {
                checked,
                expression,
                ty,
            } => AnalysisTerm::Annotation {
                checked,
                term: Box::new((*expression).into()),
                ty: Box::new((*ty).into()),
            },
            term::Term::Primitive(_) => todo!(),
        }
    }
}
