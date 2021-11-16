use std::{
    any::TypeId,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::edit::zipper::{CompressedTerm, Term};

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CompressedWord {
    data: Vec<bool>,
}

impl CompressedWord {
    fn new(data: Vec<bool>) -> Self {
        CompressedWord { data }
    }
}

impl<T: Zero + Clone> CompressedTerm<T> for CompressedWord {
    fn expand(&self) -> Term<T> {
        let mut term = Term::Reference("Word::empty".into(), T::zero());

        let high = Term::Reference("Word::high".into(), T::zero());

        let low = Term::Reference("Word::low".into(), T::zero());

        for (idx, bit) in self.data.iter().enumerate() {
            let call = if *bit { &high } else { &low };

            let call = Term::Application {
                function: Box::new(call.clone()),
                argument: Box::new(Term::Compressed(Box::new(CompressedSize::new(idx)))),
                erased: true,
                annotation: T::zero(),
            };

            term = Term::Application {
                function: Box::new(call),
                argument: Box::new(term),
                erased: false,
                annotation: T::zero(),
            }
        }

        term
    }

    fn box_clone(&self) -> Box<dyn CompressedTerm<T>> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "~lit Word {:?}",
            self.data
                .iter()
                .map(|bit| if *bit { "1" } else { "0" })
                .collect::<Vec<_>>()
                .join("")
        )
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        let mut data = hasher.finish().to_be_bytes().to_vec();
        data.extend(bincode::serialize(&self).unwrap());
        data
    }

    fn concrete_ty(&self) -> Option<Term<T>> {
        Some(Term::Application {
            erased: true,
            annotation: T::zero(),
            function: Box::new(Term::Reference("Word".into(), T::zero())),
            argument: Box::new(Term::Compressed(Box::new(CompressedSize::new(
                self.data.len(),
            )))),
        })
    }

    fn annotation(&self) -> T {
        T::zero()
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CompressedSize {
    size: usize,
}

impl CompressedSize {
    pub fn new(size: usize) -> CompressedSize {
        CompressedSize { size }
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for () {
    fn zero() -> Self {
        ()
    }
}

impl<T> Zero for Option<T> {
    fn zero() -> Self {
        None
    }
}

impl<T: Zero + Clone> CompressedTerm<T> for CompressedSize {
    fn expand(&self) -> Term<T> {
        let mut term = Term::Reference("Size::zero".into(), T::zero());

        if self.size > 0 {
            let succ = Term::Reference("Size::succ".into(), T::zero());

            for _ in 0..self.size {
                term = Term::Application {
                    function: Box::new(succ.clone()),
                    argument: Box::new(term),
                    erased: false,
                    annotation: T::zero(),
                }
            }
        }

        term
    }

    fn box_clone(&self) -> Box<dyn CompressedTerm<T>> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "~lit Size {:?}", self.size)
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        let mut data = hasher.finish().to_be_bytes().to_vec();
        data.extend(bincode::serialize(&self).unwrap());
        data
    }

    fn concrete_ty(&self) -> Option<Term<T>> {
        Some(Term::Reference("Size".into(), T::zero()))
    }

    fn annotation(&self) -> T {
        T::zero()
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CompressedChar {
    char: char,
}

impl CompressedChar {
    fn new(char: char) -> CompressedChar {
        CompressedChar { char }
    }
}

impl<T: Zero + Clone> CompressedTerm<T> for CompressedChar {
    fn expand(&self) -> Term<T> {
        let character = (self.char as u32).to_be_bytes();
        let mut bits = vec![];
        for byte in character {
            for bit in 0..8u8 {
                if ((1 << bit) & byte) != 0 {
                    bits.push(true);
                } else {
                    bits.push(false);
                }
            }
        }

        Term::Application {
            function: Box::new(Term::Reference("Char::new".into(), T::zero())),
            argument: Box::new(Term::Compressed(Box::new(CompressedWord::new(bits)))),
            erased: false,
            annotation: T::zero(),
        }
    }

    fn box_clone(&self) -> Box<dyn CompressedTerm<T>> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "~lit Char {:?}", self.char)
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        let mut data = hasher.finish().to_be_bytes().to_vec();
        data.extend(bincode::serialize(&self).unwrap());
        data
    }

    fn concrete_ty(&self) -> Option<Term<T>> {
        Some(Term::Reference("Char".into(), T::zero()))
    }

    fn annotation(&self) -> T {
        T::zero()
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}

pub fn make_vector<T: Zero + Clone>(ty: Term<T>, data: Vec<Term<T>>) -> Term<T> {
    let mut term = Term::Reference("Vector::nil".into(), T::zero());

    term = Term::Application {
        function: Box::new(term),
        argument: Box::new(ty.clone()),
        erased: true,
        annotation: T::zero(),
    };

    let mut cons = Term::Reference("Vector::cons".into(), T::zero());

    cons = Term::Application {
        function: Box::new(cons),
        argument: Box::new(ty),
        erased: true,
        annotation: T::zero(),
    };

    for (idx, element) in data.into_iter().rev().enumerate() {
        let call = Term::Application {
            function: Box::new(cons.clone()),
            argument: Box::new(Term::Compressed(Box::new(CompressedSize::new(idx)))),
            erased: true,
            annotation: T::zero(),
        };

        let call = Term::Application {
            function: Box::new(call),
            argument: Box::new(element),
            erased: false,
            annotation: T::zero(),
        };

        term = Term::Application {
            function: Box::new(call),
            argument: Box::new(term),
            erased: false,
            annotation: T::zero(),
        };
    }

    term
}

#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct CompressedString {
    data: String,
}

impl CompressedString {
    pub fn new(data: String) -> CompressedString {
        CompressedString { data }
    }
}

impl<T: Zero + Clone> CompressedTerm<T> for CompressedString {
    fn expand(&self) -> Term<T> {
        Term::Application {
            function: Box::new(Term::Application {
                erased: true,
                function: Box::new(Term::Reference("String::new".into(), T::zero())),
                argument: Box::new(Term::Compressed(Box::new(CompressedSize::new(
                    self.data.len(),
                )))),
                annotation: T::zero(),
            }),
            argument: Box::new(make_vector(
                Term::Reference("Char".into(), T::zero()),
                self.data
                    .chars()
                    .map(|character| Term::Compressed(Box::new(CompressedChar::new(character))))
                    .collect(),
            )),
            annotation: T::zero(),
            erased: false,
        }
    }

    fn box_clone(&self) -> Box<dyn CompressedTerm<T>> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "~lit String {}", self.data)
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        let mut data = hasher.finish().to_be_bytes().to_vec();
        data.extend(bincode::serialize(&self).unwrap());
        data
    }

    fn concrete_ty(&self) -> Option<Term<T>> {
        Some(Term::Application {
            erased: true,
            annotation: T::zero(),
            function: Box::new(Term::Reference("String".into(), T::zero())),
            argument: Box::new(Term::Compressed(Box::new(CompressedSize::new(
                self.data.len(),
            )))),
        })
    }

    fn annotation(&self) -> T {
        T::zero()
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}
