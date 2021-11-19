use crate::edit::{
    dynamic::abst::controls::Zero,
    zipper::{Cursor, Path},
};

use super::AnalysisTerm;

impl<T> AnalysisTerm<T> {
    pub(crate) fn substitute_in(&mut self, variable: usize, term: &AnalysisTerm<T>, shift: bool)
    where
        T: Clone + Zero,
    {
        use AnalysisTerm::*;

        match self {
            Variable(idx, _) => {
                if variable == *idx {
                    *self = term.clone();
                } else if *idx > variable {
                    if shift {
                        *idx -= 1;
                    }
                }
            }
            Lambda { body, .. } => {
                let mut term = term.clone();
                term.shift_top();
                body.substitute_in(variable + 1, &term, shift);
            }
            Application {
                function, argument, ..
            } => {
                function.substitute_in(variable, term, shift);
                argument.substitute_in(variable, term, shift);
            }
            Put(expr, _) => {
                expr.substitute_in(variable, term, shift);
            }
            Duplication {
                body, expression, ..
            } => {
                expression.substitute_in(variable, term, shift);
                let mut term = term.clone();
                term.shift_top();
                body.substitute_in(variable + 1, &term, shift);
            }
            Reference(_, _) | Universe(_) | Hole(_) => {}

            Wrap(expr, _) => expr.substitute_in(variable, term, shift),
            Annotation {
                term: expression,
                ty,
                ..
            } => {
                expression.substitute_in(variable, term, shift);
                ty.substitute_in(variable, term, shift);
            }
            Function {
                argument_type,
                return_type,
                ..
            } => {
                argument_type.substitute_in(variable, term, shift);

                let mut term = term.clone();
                term.shift_top_by(2);
                return_type.substitute_in(variable + 2, &term, shift);
            }
            Compressed(data) => {
                let data = Cursor::<()>::from_term_and_path(data.expand(), Path::Top);
                let data: AnalysisTerm<Option<()>> = data.into();
                *self = data.map_annotation(&mut |data| T::zero());
                self.substitute_in(variable, term, shift);
            }
        }
    }

    pub fn substitute_top_in(&mut self, term: &AnalysisTerm<T>)
    where
        T: Clone + Zero,
    {
        self.substitute_in(0, term, true)
    }

    pub(crate) fn substitute_top_in_unshifted(&mut self, term: &AnalysisTerm<T>)
    where
        T: Clone + Zero,
    {
        self.substitute_in(0, term, false)
    }

    pub(crate) fn substitute_function_in(
        &mut self,
        mut self_binding: AnalysisTerm<T>,
        argument_binding: &AnalysisTerm<T>,
    ) where
        T: Clone + Zero,
    {
        self_binding.shift_top();
        self.substitute_in(1, &self_binding, true);
        self.substitute_in(0, argument_binding, true);
    }

    pub(crate) fn substitute_function_in_unshifted(
        &mut self,
        mut self_binding: AnalysisTerm<T>,
        argument_binding: &AnalysisTerm<T>,
    ) where
        T: Clone + Zero,
    {
        self_binding.shift_top();
        self.substitute_in(1, &self_binding, true);
        self.substitute_in(0, argument_binding, false);
    }
}
