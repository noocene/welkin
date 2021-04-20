use combine::{choice, look_ahead, token, Parser, Stream};
use welkin_core::term::Term as CoreTerm;

use bumpalo::Bump;

use crate::{compiler::AbsolutePath, parser::util::string};

mod match_arms;
use match_arms::match_block;
pub(crate) use match_arms::{Arm, Match, Section};

use super::Context;

pub fn block_keyword<Input>() -> impl Parser<Input, Output = &'static str>
where
    Input: Stream<Token = char>,
{
    token('~').with(look_ahead(choice([string("match")])))
}

#[derive(Debug, Clone)]
pub enum Block<'a> {
    AbsoluteCore(CoreTerm<AbsolutePath>),
    Match(Match<'a>),
}

pub fn block<'a, Input>(context: Context, bump: &'a Bump) -> impl Parser<Input, Output = Block<'a>>
where
    Input: Stream<Token = char>,
{
    block_keyword().with(choice!(
        string("match").with(match_block(context.clone(), bump))
    ))
}
