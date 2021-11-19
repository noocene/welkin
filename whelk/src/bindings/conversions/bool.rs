use crate::bindings::w;

impl From<w::Bool> for bool {
    fn from(bool: w::Bool) -> Self {
        match bool {
            w::Bool::r#true => true,
            w::Bool::r#false => false,
        }
    }
}
