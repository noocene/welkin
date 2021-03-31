use std::fmt::{self, Debug};

use welkin_core::term::{Index, Show};

use crate::parser::{Ident, Path};

pub mod item;
pub mod term;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct AbsolutePath(pub Vec<String>);

impl Show for AbsolutePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            write!(f, "::{}", item)?;
        }
        Ok(())
    }
}

impl Debug for AbsolutePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Show::fmt(self, f)
    }
}

pub enum Resolved<T> {
    Index(Index),
    Canonicalized(T),
}

impl<T> Resolved<T> {
    pub fn unwrap_index(self) -> Index {
        match self {
            Resolved::Index(idx) => idx,
            _ => panic!("attempted to unwrap non-index as index"),
        }
    }
}

pub trait Resolve<T> {
    type Absolute;
    type Error: Debug;
    type Unit;

    fn resolve(&self, item: &T) -> Result<Resolved<Self::Absolute>, Self::Error>;
    fn canonicalize(&self, item: T) -> Self::Absolute;
    fn descend(&self, item: Option<Self::Unit>) -> Self;
    fn ascend(&self) -> Self;
    fn proceed(&self) -> Self;
}

#[must_use]
#[derive(Debug)]
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
            self.0
                .iter()
                .rev()
                .enumerate()
                .find(|(_, a)| a.as_ref() == Some(ident))
                .map(|id| Resolved::Index(Index(id.0)))
                .unwrap_or_else(|| Resolved::Canonicalized(self.canonicalize(item.clone())))
        } else {
            Resolved::Canonicalized(AbsolutePath(
                item.0.clone().into_iter().map(|a| a.0).collect(),
            ))
        })
    }

    fn canonicalize(&self, path: Path) -> Self::Absolute {
        AbsolutePath(path.0.into_iter().map(|a| a.0).collect())
    }

    fn descend(&self, item: Option<Self::Unit>) -> Self {
        let mut this = LocalResolver(self.0.clone());
        this.0.push(item);
        this
    }

    fn ascend(&self) -> Self {
        let mut this = LocalResolver(self.0.clone());
        this.0.pop();
        this
    }

    fn proceed(&self) -> Self {
        LocalResolver(self.0.clone())
    }
}
