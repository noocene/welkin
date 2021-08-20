use welkin_core::term::Term as CoreTerm;

use parser::{
    term::{Block, Literal, Term},
    util::BumpBox,
    AbsolutePath, BumpString, BumpVec, Ident, Path,
};

mod match_arms;

use super::{Resolve, Resolved};

pub trait Compile<T> {
    type Relative;
    type Absolute;
    type Unit;

    fn compile<R: Resolve<Self::Relative, Absolute = Self::Absolute, Unit = Self::Unit>>(
        self,
        resolver: R,
    ) -> CoreTerm<T>;
}

impl<'a, U, T: Compile<U> + Clone> Compile<U> for BumpBox<'a, T> {
    type Relative = T::Relative;
    type Absolute = T::Absolute;
    type Unit = T::Unit;

    fn compile<R: Resolve<Self::Relative, Absolute = Self::Absolute, Unit = Self::Unit>>(
        self,
        resolver: R,
    ) -> CoreTerm<U> {
        self.clone_inner().compile(resolver)
    }
}

impl<'a> Compile<AbsolutePath> for Term<'a> {
    type Relative = Path<'a>;
    type Absolute = AbsolutePath;
    type Unit = Ident<'a>;

    fn compile<R: Resolve<Path<'a>, Unit = Ident<'a>, Absolute = AbsolutePath>>(
        self,
        resolver: R,
    ) -> CoreTerm<AbsolutePath> {
        match self {
            Term::Universe => CoreTerm::Universe,
            Term::Lambda {
                argument,
                body,
                erased,
            } => CoreTerm::Lambda {
                erased,
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
                erased,
                return_type,
                self_binding,
            } => {
                let argument_type = Box::new(argument_type.compile(resolver.proceed()));
                let return_type = Box::new(
                    return_type.compile(resolver.descend(self_binding).descend(argument_binding)),
                );
                CoreTerm::Function {
                    argument_type,
                    return_type,
                    erased,
                }
            }
        }
    }
}

impl<'a> Compile<AbsolutePath> for Block<'a> {
    type Relative = Path<'a>;
    type Absolute = AbsolutePath;
    type Unit = Ident<'a>;

    fn compile<R: Resolve<Path<'a>, Unit = Ident<'a>, Absolute = AbsolutePath>>(
        self,
        resolver: R,
    ) -> CoreTerm<AbsolutePath> {
        match self {
            Block::AbsoluteCore(core) => core,
            Block::Match(m) => m.compile(resolver),
            Block::Literal(l, bump) => match l {
                Literal::Word(word) => {
                    let mut term = Term::Reference(Path(BumpVec::binary_in(
                        Ident(BumpString::from_str("Word", bump)),
                        Ident(BumpString::from_str("empty", bump)),
                        bump,
                    )));

                    let high = Term::Reference(Path(BumpVec::binary_in(
                        Ident(BumpString::from_str("Word", bump)),
                        Ident(BumpString::from_str("high", bump)),
                        bump,
                    )));

                    let low = Term::Reference(Path(BumpVec::binary_in(
                        Ident(BumpString::from_str("Word", bump)),
                        Ident(BumpString::from_str("low", bump)),
                        bump,
                    )));

                    for (idx, bit) in word.into_iter().enumerate() {
                        let call = if bit { &high } else { &low };

                        let call = Term::Application {
                            function: BumpBox::new_in(call.clone(), bump),
                            arguments: BumpVec::unary_in(
                                Term::Block(Block::Literal(Literal::Size(idx), bump)),
                                bump,
                            ),
                            erased: true,
                        };

                        term = Term::Application {
                            function: BumpBox::new_in(call, bump),
                            arguments: BumpVec::unary_in(term, bump),
                            erased: false,
                        }
                    }

                    term.compile(resolver)
                }
                Literal::Size(size) => {
                    let mut term = Term::Reference(Path(BumpVec::binary_in(
                        Ident(BumpString::from_str("Size", bump)),
                        Ident(BumpString::from_str("zero", bump)),
                        bump,
                    )));

                    if size > 0 {
                        let succ = Term::Reference(Path(BumpVec::binary_in(
                            Ident(BumpString::from_str("Size", bump)),
                            Ident(BumpString::from_str("succ", bump)),
                            bump,
                        )));

                        for _ in 0..size {
                            term = Term::Application {
                                function: BumpBox::new_in(succ.clone(), bump),
                                arguments: BumpVec::unary_in(term, bump),
                                erased: false,
                            };
                        }
                    }

                    term.compile(resolver)
                }
                Literal::Char(character) => {
                    let character = (character as u32).to_be_bytes();
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
                        function: BumpBox::new_in(
                            Term::Reference(Path(BumpVec::binary_in(
                                Ident(BumpString::from_str("Char", bump)),
                                Ident(BumpString::from_str("new", bump)),
                                bump,
                            ))),
                            bump,
                        ),
                        arguments: BumpVec::unary_in(
                            Term::Block(Block::Literal(Literal::Word(bits), bump)),
                            bump,
                        ),
                        erased: false,
                    }
                    .compile(resolver)
                }
            },
        }
    }
}
