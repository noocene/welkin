use bumpalo::Bump;
use combine::{
    any, choice, dispatch, many, many1, none_of, one_of,
    parser::{
        char::{digit, spaces},
        combinator::recognize,
        repeat::escaped,
    },
    unexpected_any, Parser, Stream,
};

use crate::{
    comma_separated, term,
    term::Context,
    util::{bare_ident, bare_token, delimited},
    BumpBox, BumpVec, Term,
};

use super::Block;

#[derive(Debug, Clone)]
pub enum Literal<'a> {
    Size(usize),
    Char(char),
    Word(Vec<bool>),
    Vector {
        ty: BumpBox<'a, Term<'a>>,
        elements: BumpVec<'a, Term<'a>>,
    },
    String(String),
}

pub fn literal<Input>(ctx: Context, bump: &Bump) -> impl Parser<Input, Output = Block>
where
    Input: Stream<Token = char>,
{
    spaces().with(bare_ident(bump)).then(move |a| {
        let ctx = ctx.clone();
        let bump = bump.clone();
        dispatch!(
            a.0.data.as_str();
            "Size" => {
                spaces().with(many1(digit())).map(|a: String| a.parse::<usize>().unwrap()).map(Literal::Size).skip(spaces())
            },
            "Word" => {
                spaces().with(many(choice([bare_token('0'), bare_token('1')]).map(|bit| match bit {
                    '0' => false,
                    '1' => true,
                    _ => panic!(),
                }))).map(Literal::Word)
            },
            "Char" => {
                spaces().with(delimited('\'','\'', any())).map(Literal::Char).skip(spaces())
            },
            "Vector" => {
                (delimited('[',']', term(ctx.clone(), bump)).skip(spaces()), delimited('[',']', comma_separated(move || term(ctx.clone(), bump), bump))).map(move |(ty, elements)| {
                    Literal::Vector {
                        ty: BumpBox::new_in(ty, bump),
                        elements
                    }
                })
            },
            "String" => {
                spaces().with(bare_token('"').with(
                    recognize(escaped(
                        many1::<String, _, _>(none_of(['\\', '"'])),
                        '\\',
                        one_of(['\\', '"'])
                    )))).map(|string: String| {
                        let mut string = string.chars();
                        string.next_back();
                        Literal::String(string.as_str().to_owned())
                    })
            },
            _ => unexpected_any("unknown literal format")
        )
    }).map(move |a| Block::Literal(a, bump))
}
