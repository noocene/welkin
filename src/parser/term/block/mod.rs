use combine::{choice, look_ahead, parser::char::spaces, token, Parser, Stream};
use welkin_core::term::{parse, Term as CoreTerm};

use crate::{
    compiler::AbsolutePath,
    parser::{
        util::{delimited, string},
        Ident, Path,
    },
};

mod match_arms;
use match_arms::match_block;
pub(crate) use match_arms::{Arm, Match, Section};

use super::{term, Context, Term};

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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        string("match").with(match_block(context.clone())),
        string("open")
            .skip(spaces())
            .with(delimited(
                '(',
                ')',
                (
                    term(context.clone()).skip(token(':')).map(Box::new),
                    term(context.clone())
                )
            ))
            .map(|(expression, ty)| {
                Block::Match(Match {
                    expression,
                    sections: vec![Section {
                        ty,
                        arms: vec![Arm {
                            introductions: vec![Ident("~open-intro".into())],
                            expression: Term::Reference(Path(vec![Ident("~open-intro".into())])),
                        }],
                    }],
                })
            })
    ))
}
