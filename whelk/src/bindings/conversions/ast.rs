use crate::{bindings::w, edit::zipper::analysis::AnalysisTerm};

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
