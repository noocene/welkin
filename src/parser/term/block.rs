use combine::{choice, parser::char::spaces, token, Parser, Stream};
use welkin_core::term::{parse, Term};

use crate::parser::util::{delimited, string};

use super::Context;

pub fn block_keyword<Input>() -> impl Parser<Input, Output = &'static str>
where
    Input: Stream<Token = char>,
{
    token('~').with(choice([string("core")]))
}

#[derive(Debug, Clone)]
pub enum Block {
    Core(Term<String>),
}

pub fn block<Input>(_: Context) -> impl Parser<Input, Output = Block>
where
    Input: Stream<Token = char>,
{
    block_keyword().then(|kw| match kw {
        "core" => spaces().with(delimited('{', '}', parse().map(Block::Core))),
        _ => panic!(),
    })
}
