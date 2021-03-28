use std::{
    borrow::Cow,
    convert::TryInto,
    fmt::{self, Debug},
};

use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use welkin_core::term::Term;

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Hash([u8; 32]);

impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(0x")?;
        for byte in &self.0 {
            write!(f, "{:X}", byte)?;
        }
        write!(f, ")")
    }
}

pub trait ReferenceHash {
    fn hash(&self) -> Cow<'_, [u8]>;
}

#[repr(u8)]
enum TermVariant {
    Variable,
    Lambda,
    Apply,
    Put,
    Duplicate,
    Reference,
    Universe,
    Function,
    Annotation,
    Wrap,
}

fn variant<T>(t: &Term<T>) -> u8 {
    use Term::*;

    (match t {
        Variable(_) => TermVariant::Variable,
        Lambda { .. } => TermVariant::Lambda,
        Apply { .. } => TermVariant::Apply,
        Put(_) => TermVariant::Put,
        Duplicate { .. } => TermVariant::Duplicate,
        Reference { .. } => TermVariant::Reference,
        Universe { .. } => TermVariant::Universe,
        Function { .. } => TermVariant::Function,
        Annotation { .. } => TermVariant::Annotation,
        Wrap { .. } => TermVariant::Wrap,
    }) as u8
}

impl ReferenceHash for Hash {
    fn hash(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
}

pub(crate) fn hash<T: ReferenceHash>(t: &Term<T>) -> Hash {
    fn hash_helper<T: ReferenceHash>(t: &Term<T>, context: &mut Context) {
        context.update(&[variant(t)]);
        use Term::*;

        match t {
            Variable(var) => context.update(&(var.value() as u32).to_be_bytes()),
            Lambda { body, erased } => {
                context.update(&[if *erased { 0 } else { 1 }]);
                hash_helper(body, context);
            }
            Apply {
                function,
                argument,
                erased,
            } => {
                context.update(&[if *erased { 0 } else { 1 }]);
                hash_helper(function, context);
                hash_helper(argument, context);
            }
            Put(term) | Wrap(term) => hash_helper(term, context),
            Duplicate { expression, body } => {
                hash_helper(expression, context);
                hash_helper(body, context);
            }
            Reference(reference) => context.update(reference.hash().as_ref()),
            Function {
                argument_type,
                return_type,
                erased,
            } => {
                context.update(&[if *erased { 0 } else { 1 }]);
                hash_helper(argument_type, context);
                hash_helper(return_type, context);
            }
            Annotation {
                checked,
                expression,
                ty,
            } => {
                context.update(&[if *checked { 0 } else { 1 }]);
                hash_helper(expression, context);
                hash_helper(ty, context);
            }
            Universe => {}
        }
    }

    let mut ctx = Context::new(&SHA256);

    hash_helper(t, &mut ctx);

    Hash(ctx.finish().as_ref().try_into().unwrap())
}
