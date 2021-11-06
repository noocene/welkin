use super::AnalysisTerm;

impl<T> AnalysisTerm<T> {
    pub fn is_complete(&self) -> bool {
        match self {
            AnalysisTerm::Lambda { body, .. } => body.is_complete(),
            AnalysisTerm::Variable(_, _) => true,
            AnalysisTerm::Application {
                function, argument, ..
            } => function.is_complete() && argument.is_complete(),
            AnalysisTerm::Put(term, _) => term.is_complete(),
            AnalysisTerm::Duplication {
                expression, body, ..
            } => expression.is_complete() && body.is_complete(),
            AnalysisTerm::Reference(_, _) => true,
            AnalysisTerm::Universe(_) => true,
            AnalysisTerm::Function {
                argument_type,
                return_type,
                ..
            } => argument_type.is_complete() && return_type.is_complete(),
            AnalysisTerm::Wrap(term, _) => term.is_complete(),
            AnalysisTerm::Hole(_) => false,
            AnalysisTerm::Annotation { term, ty, .. } => term.is_complete() && ty.is_complete(),
            AnalysisTerm::Compressed(_) => {
                // TODO actual implementation
                true
            }
        }
    }
}

impl<T> AnalysisTerm<T> {
    pub fn no_variables(&self) -> bool {
        match self {
            AnalysisTerm::Lambda { body, .. } => body.no_variables(),
            AnalysisTerm::Variable(_, _) => false,
            AnalysisTerm::Application {
                function, argument, ..
            } => function.no_variables() && argument.no_variables(),
            AnalysisTerm::Put(term, _) => term.no_variables(),
            AnalysisTerm::Duplication {
                expression, body, ..
            } => expression.no_variables() && body.no_variables(),
            AnalysisTerm::Reference(_, _) => true,
            AnalysisTerm::Universe(_) => true,
            AnalysisTerm::Function {
                argument_type,
                return_type,
                ..
            } => argument_type.no_variables() && return_type.no_variables(),
            AnalysisTerm::Wrap(term, _) => term.no_variables(),
            AnalysisTerm::Hole(_) => true,
            AnalysisTerm::Annotation { term, ty, .. } => term.no_variables() && ty.no_variables(),
            AnalysisTerm::Compressed(_) => {
                // TODO actual implementation
                true
            }
        }
    }
}
