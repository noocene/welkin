use welkin_core::term::Term;

pub mod hash;
use hash::{Hash, ReferenceHash};

pub mod definitions;
pub mod parser;

pub mod compile;

mod sealed {
    use super::Term;

    pub trait Sealed {}

    impl<T> Sealed for Term<T> {}
}

pub trait TermExt: sealed::Sealed {
    fn hash(&self) -> Hash;
}

impl<T: ReferenceHash> TermExt for Term<T> {
    fn hash(&self) -> Hash {
        hash::hash(self)
    }
}
