use std::{cell::RefCell, rc::Rc};

use crate::edit::{
    zipper::{Cursor, RefCount, Term},
    UiSection,
};

fn set(target: &Rc<RefCell<Term<(), RefCount>>>, term: Term<()>) {
    let target = &mut *target.borrow_mut();
    match (&mut *target, &term) {
        (
            Term::Function {
                return_type,
                argument_type,
                erased,
                name,
                self_name,
                annotation,
            },
            Term::Function {
                return_type: return_term,
                argument_type: argument_term,
                erased: a_erased,
                name: a_name,
                self_name: a_self_name,
                ..
            },
        ) => {
            set(return_type, return_term.as_ref().clone());
            set(argument_type, argument_term.as_ref().clone());
            *erased = *a_erased;
            *name = a_name.clone();
            *self_name = a_self_name.clone();
        }
        (_, _) => {
            *target = term.into();
        }
    }
}

impl Cursor<UiSection> {
    fn annotate_inner(&mut self, term: Term<()>) {
        set(&self.annotation().annotation, term);
    }

    pub fn annotate(&mut self, term: Term<()>) {
        self.annotate_inner(term);
    }
}
