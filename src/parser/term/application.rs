use bumpalo::Bump;
use combine::{parser, parser::combinator::Either, value, Parser, Stream};

use crate::parser::util::{comma_separated1, delimited, BumpBox};

use super::{term, Context, Term};

pub fn application<'a, Input>(
    erased: bool,
    group: Term<'a>,
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Term<'a>>
where
    Input: Stream<Token = char>,
{
    let parser = delimited(
        if erased { '[' } else { '(' },
        if erased { ']' } else { ')' },
        comma_separated1(
            {
                let context = context.clone();
                move || term(context.clone(), bump)
            },
            bump,
        ),
    )
    .map(move |arguments| Term::Application {
        erased,
        function: BumpBox::new_in(group.clone(), bump),
        arguments,
    });
    if erased {
        Either::Left(
            parser.then(move |term| {
                recurse(false, term.clone(), context.clone(), bump).or(value(term))
            }),
        )
    } else {
        Either::Right(parser)
    }
}

parser! {
    fn recurse['a, Input](erased: bool, group: Term<'a>, context: Context, bump: &'a Bump)(Input) -> Term<'a>
    where
         [ Input: Stream<Token = char> ]
    {
        application(erased.clone(), group.clone(), context.clone(), bump)
    }
}
