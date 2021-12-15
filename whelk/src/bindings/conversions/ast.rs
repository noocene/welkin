use std::convert::{TryFrom, TryInto};

use crate::{
    bindings::w,
    edit::zipper::{analysis::AnalysisTerm, Term},
};

#[derive(Debug, Clone)]
pub struct Incomplete;

impl TryFrom<Term<()>> for w::Ast {
    type Error = Incomplete;

    fn try_from(term: Term<()>) -> Result<Self, Self::Error> {
        Ok(match term {
            Term::Lambda { erased, body, .. } => w::Ast::Lambda {
                erased: erased.into(),
                body: Box::new((*body).try_into()?),
            },
            Term::Application {
                erased,
                function,
                argument,
                ..
            } => w::Ast::Application {
                erased: erased.into(),
                function: Box::new((*function).try_into()?),
                argument: Box::new((*argument).try_into()?),
            },
            Term::Put(term, _) => w::Ast::Put {
                term: Box::new((*term).try_into()?),
            },
            Term::Duplication {
                expression, body, ..
            } => w::Ast::Duplication {
                expression: Box::new((*expression).try_into()?),
                body: Box::new((*body).try_into()?),
            },
            Term::Reference(data, _) => w::Ast::Reference {
                name: w::Sized::new {
                    size: data.len().into(),
                    data: data.into(),
                },
            },
            Term::Universe(data) => w::Ast::Universe,
            Term::Function {
                erased,
                argument_type,
                return_type,
                ..
            } => w::Ast::Function {
                erased: erased.into(),
                argument_type: Box::new((*argument_type).try_into()?),
                return_type: Box::new((*return_type).try_into()?),
            },
            Term::Wrap(term, _) => w::Ast::Wrap {
                term: Box::new((*term).try_into()?),
            },
            Term::Hole(_) => Err(Incomplete)?,
            Term::Dynamic(data) => {
                let term = data.into_inner().1.expand();
                term.try_into()?
            }
            Term::Compressed(data) => {
                let term = data.expand();
                term.try_into()?
            }
        })
    }
}

impl From<w::Ast> for AnalysisTerm<()> {
    fn from(term: w::Ast) -> Self {
        match term {
            w::Ast::Lambda { r#erased, r#body } => AnalysisTerm::Lambda {
                erased: erased.into(),
                body: Box::new((*body).into()),
                name: None,
                annotation: (),
            },
            w::Ast::Variable { r#index } => AnalysisTerm::Variable(index.into(), ()),
            w::Ast::Application {
                r#erased,
                r#function,
                r#argument,
            } => AnalysisTerm::Application {
                erased: erased.into(),
                function: Box::new((*function).into()),
                argument: Box::new((*argument).into()),
                annotation: (),
            },
            w::Ast::Put { r#term } => AnalysisTerm::Put(Box::new((*term).into()), ()),
            w::Ast::Duplication {
                r#expression,
                r#body,
            } => AnalysisTerm::Duplication {
                expression: Box::new((*expression).into()),
                body: Box::new((*body).into()),
                binder: None,
                annotation: (),
            },
            w::Ast::Reference { r#name } => AnalysisTerm::Reference(
                match name {
                    w::Sized::new { data, .. } => data.into(),
                },
                (),
            ),
            w::Ast::Universe => AnalysisTerm::Universe(()),
            w::Ast::Function {
                r#erased,
                r#argument_type,
                r#return_type,
            } => AnalysisTerm::Function {
                erased: erased.into(),
                argument_type: Box::new((*argument_type).into()),
                return_type: Box::new((*return_type).into()),
                name: None,
                self_name: None,
                annotation: (),
            },
            w::Ast::Wrap { r#term } => AnalysisTerm::Wrap(Box::new((*term).into()), ()),
        }
    }
}
