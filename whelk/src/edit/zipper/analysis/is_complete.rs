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
        }
    }
}
