use std::fmt::{self, Debug};

use welkin_core::term::{Index, Show};

use crate::parser::{Ident, Path};

pub mod term;

#[derive(Debug, Clone)]
pub struct AbsolutePath(Vec<String>);

impl Show for AbsolutePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            write!(f, "::{}", item)?;
        }
        Ok(())
    }
}

pub enum Resolved<T> {
    Index(Index),
    Canonicalized(T),
}

pub trait Resolve<T> {
    type Absolute;
    type Error: Debug;
    type Unit;

    fn resolve(&self, item: &T) -> Result<Resolved<Self::Absolute>, Self::Error>;
    fn descend(&self, item: Option<Self::Unit>) -> Self;
    fn proceed(&self) -> Self;
}

pub struct LocalResolver(Vec<Option<Ident>>);

impl LocalResolver {
    pub fn new() -> Self {
        LocalResolver(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct NameError(pub String);

impl Resolve<Path> for LocalResolver {
    type Absolute = AbsolutePath;
    type Error = NameError;
    type Unit = Ident;

    fn resolve(&self, item: &Path) -> Result<Resolved<Self::Absolute>, Self::Error> {
        Ok(if item.0.len() == 1 {
            let ident = item.0.iter().next().unwrap();
            Resolved::Index(Index(
                self.0
                    .iter()
                    .rev()
                    .enumerate()
                    .find(|(_, a)| a.as_ref() == Some(ident))
                    .ok_or_else(|| NameError(ident.0.clone()))?
                    .0,
            ))
        } else {
            panic!()
        })
    }

    fn descend(&self, item: Option<Self::Unit>) -> Self {
        let mut this = LocalResolver(self.0.clone());
        this.0.push(item);
        this
    }

    fn proceed(&self) -> Self {
        LocalResolver(self.0.clone())
    }
}
