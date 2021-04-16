use combine::{parser, parser::combinator::Either, value, Parser, Stream};

use crate::parser::util::{comma_separated1, delimited};

use super::{term, Context, Term};

pub fn application<Input>(
    erased: bool,
    group: Term,
    context: Context,
) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    let parser = delimited(
        if erased { '[' } else { '(' },
        if erased { ']' } else { ')' },
        comma_separated1(term(context.clone())),
    )
    .map(move |arguments| Term::Application {
        erased,
        function: Box::new(group.clone()),
        arguments,
    });
    if erased {
        Either::Left(
            parser.then(move |term| recurse(false, term.clone(), context.clone()).or(value(term))),
        )
    } else {
        Either::Right(parser)
    }
}

parser! {
    fn recurse[Input](erased: bool, group: Term, context: Context)(Input) -> Term
    where
         [ Input: Stream<Token = char> ]
    {
        application(erased.clone(), group.clone(), context.clone())
    }
}
