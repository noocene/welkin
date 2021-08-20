use bumpalo::Bump;
use combine::{
    any, choice, dispatch, many, many1,
    parser::char::{digit, spaces},
    unexpected_any, Parser, Stream,
};

use crate::{
    term::Context,
    util::{bare_ident, bare_token, delimited},
};

use super::Block;

#[derive(Debug, Clone)]
pub enum Literal {
    Size(usize),
    Char(char),
    Word(Vec<bool>),
}

pub fn literal<Input>(_: Context, bump: &Bump) -> impl Parser<Input, Output = Block>
where
    Input: Stream<Token = char>,
{
    spaces().with(bare_ident(bump)).then(|a| {
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
            _ => unexpected_any("unknown literal format")
        )
    }).map(move |a| Block::Literal(a, bump))
}
