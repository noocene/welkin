use combine::{
    choice, look_ahead, optional, parser,
    parser::{
        char::{spaces, string as bare_string},
        combinator::Either,
    },
    sep_by1, token as bare_token, unexpected_any, value, Parser, Stream,
};

use super::{
    util::{bare_path, delimited, ident},
    Ident, Path,
};

mod application;
use application::application;
mod block;
use block::{block, block_keyword, Block};

#[derive(Debug, Clone)]
pub enum Term {
    Universe,
    Lambda {
        arguments: Vec<Ident>,
        body: Box<Term>,
    },
    Reference(Path),
    Application {
        function: Box<Term>,
        arguments: Vec<Term>,
    },
    Block(Block),
    Function {
        argument_binding: Option<Ident>,
        argument_type: Box<Term>,
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
        group.skip(spaces()).then(|group| {
            choice!(
                next_token_is('[').with(application(group.clone(), context.clone())),
                value(group)
            )
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct Context {}

impl Context {}

fn term_fragment<Input>(context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    let parser = look_ahead(block_keyword()).with(block(context.clone()).map(Term::Block));
    let parser = parser.or(recurse(context));
    parser.or(bare_token('*').with(value(Term::Universe)))
}

pub fn term<Input>(context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    sep_by1(
        spaces()
            .with(term_fragment(context).map(Box::new))
            .skip(spaces())
            .and(optional(bare_string("as").with(ident())))
            .skip(spaces()),
        bare_string("->"),
    )
    .then(move |data: Vec<_>| {
        let mut argument_types = data.into_iter().rev();
        let (return_type, return_name) = argument_types.next().unwrap();

        if return_name.is_some() {
            Either::Left(unexpected_any("name binding on return type"))
        } else {
            Either::Right(value(
                if let Some((argument_type, argument_binding)) = argument_types.next() {
                    let mut term = Term::Function {
                        argument_type,
                        return_type,
                        argument_binding,
                    };
                    while let Some((argument_type, argument_binding)) = argument_types.next() {
                        term = Term::Function {
                            argument_type,
                            argument_binding,
                            return_type: Box::new(term),
                        }
                    }
                    term
                } else {
                    *return_type
                },
            ))
        }
    })
}
