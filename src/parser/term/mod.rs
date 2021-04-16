use combine::{
    attempt, choice, look_ahead, many, optional, parser,
    parser::{
        char::{spaces, string as bare_string},
        combinator::Either,
    },
    token as bare_token, value, Parser, Stream,
};

use super::{
    util::{bare_path, delimited, ident, string},
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Universe,
    Lambda {
        argument: Ident,
        body: Box<Term>,
        erased: bool,
    },
    Reference(Path),
    Application {
        function: Box<Term>,
        erased: bool,
        arguments: Vec<Term>,
    },
    Duplicate {
        binding: Ident,
        expression: Box<Term>,
        body: Box<Term>,
    },
    Wrap(Box<Term>),
    Put(Box<Term>),
    Block(Block),
    Function {
        argument_binding: Option<Ident>,
        argument_type: Box<Term>,
        erased: bool,
        return_type: Box<Term>,
    },
}

fn next_token_is<Input>(t: char) -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
{
    look_ahead(bare_token(t))
}

fn group<Input>(context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    delimited('(', ')', term(context.clone()))
}

fn group_or_ident<Input>(context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    next_token_is('(')
        .with(group(context))
        .or(bare_path().map(Term::Reference))
}

parser! {
    fn recurse[Input](context: Context)(Input) -> Term
    where
         [ Input: Stream<Token = char> ]
    {

        let group = group_or_ident(context.clone());
        let parser = group.skip(spaces()).then(|group| {
            let choice = choice!(
                next_token_is('[').with(application(true, group.clone(), context.clone())),
                next_token_is('(').with(application(false, group.clone(), context.clone())),
                value(group.clone())
            );

            if let Term::Reference(path) = &group {
                if path.0.len() == 1 {
                    Either::Left(
                        attempt(bare_string("||>"))
                            .with(lambda(true, path.0.first().unwrap().clone(), context.clone()))
                            .or(bare_string("|>")
                            .with(lambda(false, path.0.first().unwrap().clone(), context.clone())))
                            .or(bare_token('<').with(duplicate(path.0.first().unwrap().clone(), context.clone())))
                            .or(choice)
                        )
                } else {
                    Either::Right(choice)
                }
            } else {
                Either::Right(choice)
            }
        });
        let parser = parser.or(bare_token('\'').with(term_fragment(context.clone()).map(Box::new).map(Term::Wrap)));
        parser.or(bare_token('>').with(term(context.clone()).map(Box::new).map(Term::Put)))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Context {}

impl Context {}

parser! {
    fn term_fragment[Input](context: Context)(Input) -> Term
    where
         [ Input: Stream<Token = char> ]
    {
        let parser = look_ahead(block_keyword()).with(block(context.clone()).map(Term::Block));
        let parser = parser.or(recurse(context.clone()));
        parser.or(bare_token('*').with(value(Term::Universe)))
    }
}

pub fn term<Input>(context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    spaces()
        .with(many(attempt((
            term_fragment(context.clone())
                .map(Box::new)
                .and(optional(attempt(string("~as")).with(ident()))),
            spaces()
                .with(choice!(bare_string("|->"), bare_string("->")))
                .skip(spaces()),
        ))))
        .and(term_fragment(context).map(Box::new))
        .map(move |(data, return_type): (Vec<_>, _)| {
            let mut argument_types = data.into_iter().map(|(a, b)| (a, b == "|->")).rev();

            if let Some(((argument_type, argument_binding), erased)) = argument_types.next() {
                let mut term = Term::Function {
                    argument_type,
                    return_type,
                    erased,
                    argument_binding,
                };
                while let Some(((argument_type, argument_binding), erased)) = argument_types.next()
                {
                    term = Term::Function {
                        argument_type,
                        argument_binding,
                        erased,
                        return_type: Box::new(term),
                    }
                }
                term
            } else {
                *return_type
            }
        })
}
