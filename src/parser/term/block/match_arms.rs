use crate::parser::{
    term::{term, term_fragment, Context},
    util::{
        bump_many, comma_separated, comma_separated1, delimited, ident, string, token, BumpBox,
        BumpVec,
    },
    Ident, Term,
};
use combine::{attempt, optional, parser, parser::char::spaces, Parser, Stream};

use bumpalo::Bump;

use super::Block;

#[derive(Debug, Clone)]
pub struct Arm<'a> {
    pub(crate) expression: Term<'a>,
    pub(crate) introductions: BumpVec<'a, (Ident<'a>, bool)>,
}

#[derive(Debug, Clone)]
pub struct Section<'a> {
    pub(crate) ty: Term<'a>,
    pub(crate) self_binding: Ident<'a>,
    pub(crate) arms: BumpVec<'a, Arm<'a>>,
}

#[derive(Debug, Clone)]
pub struct Match<'a> {
    pub(crate) expression: BumpBox<'a, Term<'a>>,
    pub(crate) indices: BumpVec<'a, Ident<'a>>,
    pub(crate) sections: BumpVec<'a, Section<'a>>,
}

fn match_arm<'a, Input>(context: Context, bump: &'a Bump) -> impl Parser<Input, Output = Arm<'a>>
where
    Input: Stream<Token = char>,
{
    (
        (
            ident(bump),
            optional(delimited(
                '[',
                ']',
                comma_separated(move || ident(bump).map(|a| (a, true)), bump),
            ))
            .map(move |introductions| introductions.unwrap_or_else(|| BumpVec::new_in(bump))),
            optional(delimited(
                '(',
                ')',
                comma_separated(move || ident(bump).map(|a| (a, false)), bump),
            ))
            .map(move |introductions| introductions.unwrap_or_else(|| BumpVec::new_in(bump))),
        ),
        token('=').skip(spaces()).with(term_fragment(context, bump)),
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

fn match_motive<'a, Input>(
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = (Ident, Term)>
where
    Input: Stream<Token = char>,
{
    token(':').with((ident(bump).skip(string("|>")), term(context, bump)))
}

fn match_section<'a, Input>(
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Section<'a>>
where
    Input: Stream<Token = char>,
{
    (
        bump_many(
            {
                let context = context.clone();
                move || attempt(match_arm(context.clone(), bump))
            },
            bump,
        ),
        match_motive(context.clone(), bump),
    )
        .map(|(arms, (self_binding, ty))| Section {
            arms,
            ty,
            self_binding,
        })
}

parser! {
    pub fn match_block['a, Input](context: Context, bump: &'a Bump)(Input) -> Block<'a>
    where
         [ Input: Stream<Token = char> ]
    {
        let bump = *bump;
        spaces().with((
            term_fragment(context.clone(), bump).map(move |a| BumpBox::new_in(a, bump)),
            optional(attempt(string("~with")).skip(spaces()).with(comma_separated1(move || ident(bump), bump)).skip(spaces())).map(move |a| a.unwrap_or_else(move || BumpVec::new_in(bump))),
            delimited('{','}', bump_many({
                let context = context.clone();
                move || attempt(match_section(context.clone(), bump))
            }, bump)
        )).map(|(expression, indices, sections)| {
            Block::Match(Match {
                indices,
                expression,
                sections
            })
        }))
    }
}
