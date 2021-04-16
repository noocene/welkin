use crate::parser::{
    term::{term, term_fragment, Context},
    util::{comma_separated, comma_separated1, delimited, ident, string, token},
    Ident, Term,
};
use combine::{attempt, many, optional, parser, parser::char::spaces, Parser, Stream};

use super::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arm {
    pub(crate) expression: Term,
    pub(crate) introductions: Vec<(Ident, bool)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub(crate) ty: Term,
    pub(crate) arms: Vec<Arm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub(crate) expression: Box<Term>,
    pub(crate) indices: Vec<Ident>,
    pub(crate) sections: Vec<Section>,
}

fn match_arm<Input>(context: Context) -> impl Parser<Input, Output = Arm>
where
    Input: Stream<Token = char>,
{
    (
        (
            ident(),
            optional(delimited(
                '[',
                ']',
                comma_separated(ident().map(|a| (a, true))),
            ))
            .map(|introductions| introductions.unwrap_or(vec![])),
            optional(delimited(
                '(',
                ')',
                comma_separated(ident().map(|a| (a, false))),
            ))
            .map(|introductions| introductions.unwrap_or(vec![])),
        ),
        token('=').skip(spaces()).with(term_fragment(context)),
    )
        .map(
            |((_, mut introductions, mut remaining_introductions), expression)| {
                introductions.append(&mut remaining_introductions);
                Arm {
                    expression,
                    introductions,
                }
            },
        )
}

fn match_motive<Input>(context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    token(':').with(term(context))
}

fn match_section<Input>(context: Context) -> impl Parser<Input, Output = Section>
where
    Input: Stream<Token = char>,
{
    (
        many(attempt(match_arm(context.clone()))),
        match_motive(context.clone()),
    )
        .map(|(arms, ty)| Section { arms, ty })
}

parser! {
    pub fn match_block[Input](context: Context)(Input) -> Block
    where
         [ Input: Stream<Token = char> ]
    {
        spaces().with((
            term_fragment(context.clone()).map(Box::new),
            optional(attempt(string("~with")).skip(spaces()).with(comma_separated1(ident())).skip(spaces())).map(|a| a.unwrap_or(vec![])),
            delimited('{','}', many(attempt(match_section(context.clone())))
        )).map(|(expression, indices, sections)| {
            Block::Match(Match {
                indices,
                expression,
                sections
            })
        }))
    }
}
