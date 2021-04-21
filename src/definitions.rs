use welkin_core::term::{DefinitionResult, Term, TypedDefinitions};

pub struct Null;

impl<T> TypedDefinitions<T> for Null {
    fn get_typed(&self, _: &T) -> Option<DefinitionResult<(Term<T>, Term<T>)>> {
        None
    }
}

pub struct Single<T>(pub T, pub (Term<T>, Term<T>));

impl<T: Eq> TypedDefinitions<T> for Single<T> {
    fn get_typed(&self, item: &T) -> Option<DefinitionResult<(Term<T>, Term<T>)>> {
        if item == &self.0 {
            Some(DefinitionResult::Borrowed(&self.1))
        } else {
            None
        }
    }
}
