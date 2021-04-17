use combine::{choice, look_ahead, parser::char::spaces, token, Parser, Stream};
use welkin_core::term::{parse, Term as CoreTerm};

use crate::{
    compiler::AbsolutePath,
    parser::util::{delimited, string},
};

mod match_arms;
use match_arms::match_block;
pub(crate) use match_arms::{Arm, Match, Section};

use super::Context;

pub fn block_keyword<Input>() -> impl Parser<Input, Output = &'static str>
where
    Input: Stream<Token = char>,
{
    token('~').with(look_ahead(choice([
        string("core"),
        string("match"),
        string("open"),
    ])))
}

#[derive(Debug, Clone)]
pub enum Block {
    Core(CoreTerm<String>),
    AbsoluteCore(CoreTerm<AbsolutePath>),
    Match(Match),
}

pub fn block<Input>(context: Context) -> impl Parser<Input, Output = Block>
where
    Input: Stream<Token = char>,
{
    block_keyword().with(choice!(
        string("core")
            .skip(spaces())
            .with(delimited('{', '}', parse().map(Block::Core))),
        string("match").with(match_block(context.clone()))
    ))
}
