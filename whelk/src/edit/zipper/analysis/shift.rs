use super::AnalysisTerm;

impl<T> AnalysisTerm<T> {
    pub(crate) fn shift(&mut self, replaced: usize) {
        self.shift_by(replaced, 1);
    }

    pub(crate) fn shift_by(&mut self, replaced: usize, by: isize) {
        use AnalysisTerm::*;

        match self {
            Variable(index, _) => {
                if !(*index < replaced) {
                    if by > 0 {
                        *index += by as usize;
                    } else {
                        *index -= by.abs() as usize;
                    }
                }
            }
            Lambda { body, .. } => body.shift_by(replaced + 1, by),
            Application {
                function, argument, ..
            } => {
                function.shift_by(replaced, by);
                argument.shift_by(replaced, by);
            }
            Put(term, _) => {
                term.shift_by(replaced, by);
            }
            Duplication {
                expression, body, ..
            } => {
                expression.shift_by(replaced, by);
                body.shift_by(replaced + 1, by);
            }
            Reference(_, _) | Universe(_) | Hole(_) => {}

            Wrap(term, _) => term.shift_by(replaced, by),
            Annotation { term, ty, .. } => {
                term.shift_by(replaced, by);
                ty.shift_by(replaced, by);
            }
            Function {
                argument_type,
                return_type,
                ..
            } => {
                argument_type.shift_by(replaced, by);
                return_type.shift_by(replaced + 2, by);
            }
            Compressed(_) => {
                // TODO shift compressed term?
            }
        }
    }

    pub(crate) fn shift_top_by(&mut self, by: isize) {
        self.shift_by(0, by)
    }

    pub(crate) fn shift_top(&mut self) {
        self.shift_top_by(1);
    }
}
