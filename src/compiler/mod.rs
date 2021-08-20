use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use bumpalo::Bump;
use welkin_core::term::{
    alloc::Allocator, DefinitionResult, Index, Primitives, Show, Term as CoreTerm, TypedDefinitions,
};

use parser::{AbsolutePath, Ident, Path};

pub mod item;
pub mod term;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct BumpPath<'a>(pub bumpalo::collections::Vec<'a, bumpalo::collections::String<'a>>);

pub struct DefinitionConverter<
    'a,
    T,
    U: Primitives<T> + Primitives<P>,
    A: Allocator<T, U> + Allocator<P, U>,
    D: TypedDefinitions<T, U, A>,
    P,
    F: Fn(P) -> T,
    G: Fn(T) -> P,
> {
    definitions: &'a D,
    marker: PhantomData<(P, U)>,
    alloc: &'a A,
    forward: F,
    backward: G,
}

impl<
        'a,
        T,
        U: Primitives<T> + Primitives<P>,
        A: Allocator<T, U> + Allocator<P, U>,
        D: TypedDefinitions<T, U, A>,
        P,
        F: Fn(P) -> T,
        G: Fn(T) -> P,
    > DefinitionConverter<'a, T, U, A, D, P, F, G>
{
    pub fn new(definitions: &'a D, forward: F, backward: G, alloc: &'a A) -> Self {
        DefinitionConverter {
            definitions,
            marker: PhantomData,
            alloc,
            forward,
            backward,
        }
    }
}

impl<
        'a,
        T: Clone,
        U: Primitives<T> + Primitives<P> + Clone,
        A: Allocator<T, U> + Allocator<P, U>,
        D: TypedDefinitions<T, U, A>,
        P: Clone,
        F: Fn(P) -> T,
        G: Fn(T) -> P,
    > TypedDefinitions<P, U, A> for DefinitionConverter<'a, T, U, A, D, P, F, G>
{
    fn get_typed(
        &self,
        name: &P,
    ) -> Option<DefinitionResult<(CoreTerm<P, U, A>, CoreTerm<P, U, A>)>> {
        Some({
            let data = self.definitions.get_typed(&(self.forward)(name.clone()))?;
            let (ty, term) = data.as_ref();
            DefinitionResult::Owned((
                self.alloc
                    .copy(ty)
                    .map_reference_in(|a| CoreTerm::Reference((self.backward)(a)), self.alloc),
                self.alloc
                    .copy(term)
                    .map_reference_in(|a| CoreTerm::Reference((self.backward)(a)), self.alloc),
            ))
        })
    }
}

impl<'a> Show for BumpPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            write!(f, "::{}", item)?;
        }
        Ok(())
    }
}

impl<'a> Debug for BumpPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Show::fmt(self, f)
    }
}

impl<'a> BumpPath<'a> {
    pub fn new_in(path: AbsolutePath, alloc: &'a Bump) -> Self {
        BumpPath(bumpalo::collections::Vec::from_iter_in(
            path.0
                .into_iter()
                .map(|a| bumpalo::collections::String::from_str_in(&a, alloc)),
            alloc,
        ))
    }

    pub fn reallocate_in<'b>(self, alloc: &'b Bump) -> BumpPath<'b> {
        BumpPath(bumpalo::collections::Vec::from_iter_in(
            self.0
                .into_iter()
                .map(|a| bumpalo::collections::String::from_str_in(&a, alloc)),
            alloc,
        ))
    }

    pub fn reallocating_copy_in<'b>(&self, alloc: &'b Bump) -> BumpPath<'b> {
        BumpPath(bumpalo::collections::Vec::from_iter_in(
            self.0
                .iter()
                .map(|a| bumpalo::collections::String::from_str_in(&a, alloc)),
            alloc,
        ))
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
    fn resolve_unit(&self, item: &Self::Unit) -> Result<Resolved<Self::Absolute>, Self::Error>;
    fn canonicalize(&self, item: T) -> Self::Absolute;
    fn descend(&self, item: Option<Self::Unit>) -> Self;
    fn ascend(&self) -> Self;
    fn proceed(&self) -> Self;
}

#[must_use]
#[derive(Debug)]
pub struct LocalResolver<'a>(Vec<Option<Ident<'a>>>);

impl<'a> LocalResolver<'a> {
    pub fn new() -> Self {
        LocalResolver(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct NameError(pub String);

impl<'a> Resolve<Path<'a>> for LocalResolver<'a> {
    type Absolute = AbsolutePath;
    type Error = NameError;
    type Unit = Ident<'a>;

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
                item.0
                    .clone()
                    .into_iter()
                    .map(|a| a.0.to_string())
                    .collect(),
            ))
        })
    }

    fn canonicalize(&self, path: Path) -> Self::Absolute {
        AbsolutePath(path.0.into_iter().map(|a| a.0.to_string()).collect())
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

    fn resolve_unit(&self, item: &Self::Unit) -> Result<Resolved<Self::Absolute>, Self::Error> {
        Ok(self
            .0
            .iter()
            .rev()
            .enumerate()
            .find(|(_, a)| a.as_ref() == Some(item))
            .map(|id| Resolved::Index(Index(id.0)))
            .unwrap_or_else(|| {
                Resolved::Canonicalized(AbsolutePath(vec![item.0.clone().to_string()]))
            }))
    }
}
