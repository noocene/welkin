use welkin_core::term::Term as CoreTerm;

use super::Resolve;
use parser::AbsolutePath;

use std::fmt::Debug;

mod data;

pub trait Compile<T> {
    type Relative;
    type Absolute;
    type Unit;

    fn compile<R: Debug + Resolve<Self::Relative, Absolute = Self::Absolute, Unit = Self::Unit>>(
        self,
        resolver: R,
    ) -> Vec<(AbsolutePath, CoreTerm<T>, CoreTerm<T>)>;
}
