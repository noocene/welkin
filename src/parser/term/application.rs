use combine::{optional, token as bare_token, Parser, Stream};

use crate::parser::util::{comma_separated1, delimited};

use super::{term, Context, Term};

pub fn application<Input>(group: Term, context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    delimited(
        '[',
        ']',
        optional(bare_token('.')).and(comma_separated1(term(context.clone()))),
    )
    .map(move |(erased, arguments)| Term::Application {
        erased: erased.is_some(),
        function: Box::new(group.clone()),
        arguments,
    })
}
