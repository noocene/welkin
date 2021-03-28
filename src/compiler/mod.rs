use std::{
    convert::Infallible,
    fmt::{self, Debug},
};

use welkin_core::term::{Index, Show, Term as CoreTerm};

use crate::parser::{
    term::{Block, Term},
    Ident, Path,
};

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

pub trait Compile<T> {
    type Relative;
    type Absolute;
    type Unit;

    fn compile<R: Resolve<Self::Relative, Absolute = Self::Absolute, Unit = Self::Unit>>(
        self,
        resolver: R,
    ) -> CoreTerm<T>;
}

impl Compile<AbsolutePath> for Term {
    type Relative = Path;
    type Absolute = AbsolutePath;
    type Unit = Ident;

    fn compile<R: Resolve<Path, Unit = Ident, Absolute = AbsolutePath>>(
        self,
        resolver: R,
    ) -> CoreTerm<AbsolutePath> {
        match self {
            Term::Universe => CoreTerm::Universe,
            Term::Lambda { argument, body } => CoreTerm::Lambda {
                erased: false,
                body: Box::new(body.compile(resolver.descend(Some(argument)))),
            },
            Term::Reference(path) => match resolver.resolve(&path).unwrap() {
                Resolved::Index(i) => CoreTerm::Variable(i),
                Resolved::Canonicalized(path) => CoreTerm::Reference(path),
            },
            Term::Application {
                function,
                arguments,
            } => {
                let function = Box::new(function.compile(resolver.proceed()));
                let mut arguments = arguments.into_iter();
                let argument = Box::new(arguments.next().unwrap().compile(resolver.proceed()));
                let mut term = CoreTerm::Apply {
                    function,
                    argument,
                    erased: false,
                };
                while let Some(argument) = arguments.next() {
                    term = CoreTerm::Apply {
                        function: Box::new(term),
                        erased: false,
                        argument: Box::new(argument.compile(resolver.proceed())),
                    };
                }
                term
            }
            Term::Duplicate {
                binding,
                expression,
                body,
            } => {
                let expression = Box::new(expression.compile(resolver.proceed()));
                let body = Box::new(body.compile(resolver.descend(Some(binding))));
                CoreTerm::Duplicate { expression, body }
            }
            Term::Wrap(term) => CoreTerm::Wrap(Box::new(term.compile(resolver))),
            Term::Put(ty) => CoreTerm::Put(Box::new(ty.compile(resolver))),
            Term::Block(block) => block.compile(resolver),
            Term::Function {
                argument_binding,
                argument_type,
                return_type,
            } => {
                let argument_type = Box::new(argument_type.compile(resolver.proceed()));
                let return_type =
                    Box::new(return_type.compile(resolver.descend(None).descend(argument_binding)));
                CoreTerm::Function {
                    argument_type,
                    return_type,
                    erased: false,
                }
            }
        }
    }
}

impl Compile<AbsolutePath> for Block {
    type Relative = Path;
    type Absolute = AbsolutePath;
    type Unit = Ident;

    fn compile<R: Resolve<Path, Unit = Ident, Absolute = AbsolutePath>>(
        self,
        resolver: R,
    ) -> CoreTerm<AbsolutePath> {
        match self {
            Block::Core(core) => core
                .try_map_reference(|a| {
                    Ok::<_, Infallible>(match resolver.resolve(&Path(vec![Ident(a)])).unwrap() {
                        Resolved::Canonicalized(reference) => CoreTerm::Reference(reference),
                        Resolved::Index(idx) => CoreTerm::Variable(idx),
                    })
                })
                .unwrap(),
        }
    }
}
