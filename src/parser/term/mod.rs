use combine::{
    attempt, choice, look_ahead, optional, parser,
    parser::{
        char::{spaces, string as bare_string},
        combinator::Either,
    },
    token as bare_token, value, Parser, Stream,
};

use bumpalo::Bump;

use super::{
    util::{bare_path, bump_many, delimited, ident, string, BumpBox, BumpVec},
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
    fn recurse['a, Input](context: Context, bump: &'a Bump)(Input) -> Term<'a>
    where
         [ Input: Stream<Token = char> ]
    {

        let group = group_or_ident(context.clone(), bump);
        let parser = group.skip(spaces()).then(|group| {
            let choice = choice!(
                next_token_is('[').with(application(true, group.clone(), context.clone(), bump)),
                next_token_is('(').with(application(false, group.clone(), context.clone(), bump)),
                value(group.clone())
            );

            if let Term::Reference(path) = &group {
                if path.0.len() == 1 {
                    Either::Left(
                        attempt(bare_string("||>"))
                            .with(lambda(true, path.0.first().unwrap().clone(), context.clone(), bump))
                            .or(attempt(bare_string("|>"))
                            .with(lambda(false, path.0.first().unwrap().clone(), context.clone(), bump)))
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
        let parser = parser.or(bare_token('\'').with(term_fragment(context.clone(), bump).map(|a| BumpBox::new_in(a, bump)).map(Term::Wrap)));
        parser.or(bare_token('>').with(term(context.clone(), bump).map(|a| BumpBox::new_in(a, bump)).map(Term::Put)))
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
        .with(bump_many(
            {
                let context = context.clone();
                move || {
                    attempt((
                        term_fragment(context.clone(), bump)
                            .map(move |a| BumpBox::new_in(a, bump))
                            .and(optional(attempt(string("~as")).with(ident(bump)))),
                        spaces()
                            .with(
                                attempt(choice!(bare_string("|->"), bare_string("->")))
                                    .map(|a| (None, a == "|->"))
                                    .or(bare_string("|-")
                                        .with(ident(bump))
                                        .skip(bare_string("->"))
                                        .map(|a| (Some(a), true))),
                            )
                            .skip(spaces()),
                    ))
                }
            },
            bump,
        ))
        .and(term_fragment(context, bump).map(move |a| BumpBox::new_in(a, bump)))
        .map(move |(data, return_type): (BumpVec<'_, _>, _)| {
            let mut argument_types = data.into_iter().rev();

            if let Some(((argument_type, argument_binding), (self_binding, erased))) =
                argument_types.next()
            {
                let mut term = Term::Function {
                    argument_type,
                    return_type,
                    self_binding,
                    erased,
                    argument_binding,
                };
                while let Some(((argument_type, argument_binding), (self_binding, erased))) =
                    argument_types.next()
                {
                    term = Term::Function {
                        argument_type,
                        argument_binding,
                        self_binding,
                        erased,
                        return_type: BumpBox::new_in(term, bump),
                    }
                }
                term
            } else {
                return_type.clone_inner()
            }
        })
}
