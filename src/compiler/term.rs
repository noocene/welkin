use std::convert::Infallible;

use welkin_core::term::Term as CoreTerm;

use crate::parser::{
    term::{Block, Term},
    Ident, Path,
};

use super::{AbsolutePath, Resolve, Resolved};

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
                erased,
            } => {
                let function = Box::new(function.compile(resolver.proceed()));
                let mut arguments = arguments.into_iter();
                let argument = Box::new(arguments.next().unwrap().compile(resolver.proceed()));
                let mut term = CoreTerm::Apply {
                    function,
                    argument,
                    erased: erased,
                };
                while let Some(argument) = arguments.next() {
                    term = CoreTerm::Apply {
                        function: Box::new(term),
                        erased: erased,
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
