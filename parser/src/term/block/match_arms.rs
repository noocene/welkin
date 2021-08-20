use crate::{
    term::{term, term_fragment, Context},
    util::{
        bare_ident, bump_many, comma_separated, comma_separated1, delimited, ident, string, token,
        BumpBox, BumpVec,
    },
    Ident, Term,
};
use combine::{
    optional, parser,
    parser::char::{spaces, string as bare_string},
    token as bare_token, Parser, Stream,
};

use bumpalo::Bump;

use super::Block;

#[derive(Debug, Clone)]
pub struct Arm<'a> {
    pub expression: Term<'a>,
    pub introductions: BumpVec<'a, (Ident<'a>, bool)>,
}

#[derive(Debug, Clone)]
pub struct Section<'a> {
    pub ty: Term<'a>,
    pub self_binding: Ident<'a>,
    pub arms: BumpVec<'a, Arm<'a>>,
}

#[derive(Debug, Clone)]
pub struct Match<'a> {
    pub expression: BumpBox<'a, Term<'a>>,
    pub indices: BumpVec<'a, Ident<'a>>,
    pub sections: BumpVec<'a, Section<'a>>,
}

fn match_arm<'a, Input>(context: Context, bump: &'a Bump) -> impl Parser<Input, Output = Arm<'a>>
where
    Input: Stream<Token = char>,
{
    (
        (
            bare_ident(bump),
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
    bare_token(':').with((ident(bump).skip(string("|>")), term(context, bump)))
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
                move || match_arm(context.clone(), bump).skip(spaces())
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
            term_fragment(context.clone(), bump).map(move |a| BumpBox::new_in(a, bump)).skip(spaces()),
            optional(bare_string("~with").skip(spaces()).with(comma_separated1(move || ident(bump), bump)).skip(spaces())).map(move |a| a.unwrap_or_else(move || BumpVec::new_in(bump))),
            delimited('{','}', spaces().with(bump_many({
                let context = context.clone();
                move || match_section(context.clone(), bump)
            }, bump))
        )).map(|(expression, indices, sections)| {
            Block::Match(Match {
                indices,
                expression,
                sections
            })
        }))
    }
}
