use std::{cell::RefCell, iter::once, rc::Rc};

use combine::{
    choice, look_ahead, optional, parser,
    parser::{
        char::{spaces, string as bare_string},
        combinator::Either,
    },
    token as bare_token, value, Parser, Stream,
};

use bumpalo::Bump;

use super::{
    util::{bare_path, delimited, ident, BumpBox, BumpVec},
    Ident, Path,
};

mod application;
use application::application;
mod lambda;
use lambda::lambda;
mod duplicate;
use duplicate::duplicate;
mod block;
use block::{block, block_keyword};
pub(crate) use block::{Arm, Block, Match, Section};

#[derive(Debug, Clone)]
pub enum Term<'a> {
    Universe,
    Lambda {
        argument: Ident<'a>,
        body: BumpBox<'a, Term<'a>>,
        erased: bool,
    },
    Reference(Path<'a>),
    Application {
        function: BumpBox<'a, Term<'a>>,
        erased: bool,
        arguments: BumpVec<'a, Term<'a>>,
    },
    Duplicate {
        binding: Ident<'a>,
        expression: BumpBox<'a, Term<'a>>,
        body: BumpBox<'a, Term<'a>>,
    },
    Wrap(BumpBox<'a, Term<'a>>),
    Put(BumpBox<'a, Term<'a>>),
    Block(Block<'a>),
    Function {
        self_binding: Option<Ident<'a>>,
        argument_binding: Option<Ident<'a>>,
        argument_type: BumpBox<'a, Term<'a>>,
        erased: bool,
        return_type: BumpBox<'a, Term<'a>>,
    },
}

fn next_token_is<Input>(t: char) -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
{
    look_ahead(bare_token(t))
}

fn group<'a, Input>(context: Context, bump: &'a Bump) -> impl Parser<Input, Output = Term<'a>>
where
    Input: Stream<Token = char>,
{
    delimited('(', ')', term(context.clone(), bump))
}

fn group_or_ident<'a, Input>(
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Term<'a>>
where
    Input: Stream<Token = char>,
{
    next_token_is('(')
        .with(group(context, bump))
        .or(bare_path(bump).map(Term::Reference))
}

parser! {
    fn recurse['a, Input](a_context: Context, bump: &'a Bump)(Input) -> Term<'a>
    where
         [ Input: Stream<Token = char> ]
    {
        let bump = *bump;
        let group = group_or_ident(a_context.clone(), bump);
        let context = a_context.clone();
        let parser = group.skip(spaces()).then(move |group| {
            let path = if let Term::Reference(path) = &group {
                Some(path.clone())
            } else {
                None
            };
            let group = Rc::new(RefCell::new(Some(group)));
            let choice = choice!(
                next_token_is('[').with(application(true, group.clone(), context.clone(), bump)),
                next_token_is('(').with(application(false, group.clone(), context.clone(), bump)),
                value(()).then({
                    let group = group.clone(); move |_| value(group.borrow_mut().take().unwrap())
                })
            );

            if let Some(path) = path {
                if path.0.len() == 1 {
                    Either::Left(
                        bare_token('|').with(choice!(bare_token('>').with(value(false)), bare_string("|>").with(value(true))).then({
                            let context = context.clone();
                            let path = path.clone();
                            move |erased| {
                                lambda(erased, path.0.first().unwrap().clone(), context.clone(), bump)
                            }
                        }))
                            .or(bare_token('<').with(duplicate(path.0.first().unwrap().clone(), context.clone(), bump)))
                            .or(choice)
                        )
                } else {
                    Either::Right(choice)
                }
            } else {
                Either::Right(choice)
            }
        });
        let parser = parser.or(bare_token('\'').with(term_fragment(a_context.clone(), bump).map(move |a| BumpBox::new_in(a, bump)).map(Term::Wrap)));
        parser.or(bare_token('>').with(term(a_context.clone(), bump).map(move |a| BumpBox::new_in(a, bump)).map(Term::Put)))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Context {}

impl Context {}

parser! {
    fn term_fragment['a, Input](context: Context, bump: &'a Bump)(Input) -> Term<'a>
    where
         [ Input: Stream<Token = char> ]
    {
        let parser = look_ahead(block_keyword()).with(block(context.clone(), bump).map(Term::Block));
        let parser = parser.or(recurse(context.clone(), bump));
        parser.or(bare_token('*').with(value(Term::Universe)))
    }
}

pub fn term<'a, Input>(context: Context, bump: &'a Bump) -> impl Parser<Input, Output = Term<'a>>
where
    Input: Stream<Token = char>,
{
    spaces()
        .with(term_fragment(context.clone(), bump))
        .then(move |fragment| {
            let fragment = Rc::new(RefCell::new(Some(fragment)));
            let context = context.clone();
            spaces().with(parser(move |input| {
                let mut iter = (
                    optional(bare_string("~as").with(ident(bump)).skip(spaces())),
                    (
                        bare_string("->")
                            .with(value((false, None)))
                            .or(bare_string("|-").with(choice!(
                                bare_token('>').with(value((true, None))),
                                ident(bump).skip(bare_string("->")).map(|a| (true, Some(a)))
                            )))
                            .skip(spaces()),
                        term_fragment(context.clone(), bump)
                            .skip(spaces())
                            .map(|a| BumpBox::new_in(a, bump)),
                    ),
                )
                    .iter(input);

                let mut data = (&mut iter).collect::<Vec<_>>();
                let term = if let Some((
                    last_argument_binding,
                    ((last_erased, last_self_binding), term),
                )) = data.pop()
                {
                    let metas = once((last_argument_binding, last_self_binding, last_erased))
                        .chain(data.iter().rev().cloned().map(
                            |(argument_binding, ((erased, self_binding), _))| {
                                (argument_binding, self_binding, erased)
                            },
                        ));
                    let mut term = term.clone_inner();
                    for ((argument_binding, self_binding, erased), ty) in
                        metas.zip(data.iter().rev().map(|(_, (_, ty))| ty.clone()).chain(once(
                            BumpBox::new_in(fragment.borrow_mut().take().unwrap(), bump),
                        )))
                    {
                        term = Term::Function {
                            self_binding,
                            argument_binding,
                            erased,
                            argument_type: ty,
                            return_type: BumpBox::new_in(term, bump),
                        }
                    }
                    term
                } else {
                    fragment.borrow_mut().take().unwrap()
                };

                iter.into_result(term)
            }))
        })
}
