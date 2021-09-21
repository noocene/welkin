use welkin_core::term::Term;
mod size;
pub use size::Size;
mod word;
pub use word::Word;
mod char;
pub use self::char::Char;
mod vector;
pub use vector::Vector;
mod string;
pub use string::WString;
mod sized;
pub use sized::WSized;
mod whelk;
pub use whelk::Whelk;
mod box_poly;
pub use box_poly::BoxPoly;

pub trait FromWelkin: Sized {
    type Error;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error>;
}

pub trait ToWelkin {
    type Error;

    fn to_welkin(self) -> Result<Term<String>, Self::Error>;
}
