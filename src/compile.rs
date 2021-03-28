// use std::fmt::{self, Display};

// use welkin_core::term::Term as CoreTerm;

// use crate::parser::{Block, Data, Declaration, Term};

// pub trait Compile {
//     fn compile(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
// }

// impl Compile for Term {
//     fn compile(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Term::Universe => write!(f, "*"),
//             Term::Block(term) => term.compile(f),
//             Term::Lambda { arguments, body } => {
//                 for argument in arguments {
//                     write!(f, "\\{} ", argument.0)?;
//                 }
//                 body.compile(f)
//             }
//             Term::Reference(reference) => write!(f, "{}", reference.0),
//             Term::Application {
//                 function,
//                 arguments,
//             } => {
//                 write!(f, "[")?;
//                 function.compile(f)?;
//                 for argument in arguments {
//                     write!(f, " ")?;
//                     argument.compile(f)?;
//                 }
//                 write!(f, "]")
//             }
//             Term::Function {
//                 argument_type,
//                 return_type,
//             } => {
//                 write!(f, "+,:")?;
//                 argument_type.compile(f)?;
//                 write!(f, " ")?;
//                 return_type.compile(f)
//             }
//         }
//     }
// }

// impl Compile for Block {
//     fn compile(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Block::Data(Data {
//                 variants,
//                 self_type,
//             }) => {
//                 write!(f, "_data,+,:prop:")?;
//                 self_type.compile(f)?;
//                 write!(f, " *")
//             }
//         }
//     }
// }

// impl Compile for Declaration {
//     fn compile(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{} : ", self.ident.0)?;
//         self.ty.compile(f)?;
//         write!(f, " = ")?;
//         self.term.compile(f)
//     }
// }

// pub struct Compilation<T: Compile>(pub T);

// impl<T: Compile> Display for Compilation<T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.0.compile(f)
//     }
// }
