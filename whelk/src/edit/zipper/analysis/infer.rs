use std::{cell::RefCell, rc::Rc};

use crate::edit::{
    zipper::{Cursor, RefCount, Term},
    UiSection,
};

impl Cursor<UiSection> {
    pub fn infer(&self) -> Rc<RefCell<Term<(), RefCount>>> {
        let annotation = self.annotation().annotation.clone();

        {
            let a = &mut *annotation.borrow_mut();

            match self {
                Cursor::Lambda(cursor) => {
                    if let Term::Hole(_) = a {
                        *a = Term::Function {
                            erased: cursor.erased(),
                            name: cursor.name().map(str::to_owned),
                            self_name: None,
                            argument_type: Rc::new(RefCell::new(Term::Hole(()))),
                            return_type: cursor.clone().body().infer(),
                            annotation: (),
                        };
                    }
                }
                _ => {}
            }
        }

        annotation
    }
}
