use combine::{Parser, Stream};

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
    delimited(
        if erased { '{' } else { '[' },
        if erased { '}' } else { ']' },
        comma_separated1(term(context.clone())),
    )
    .map(move |arguments| Term::Application {
        erased,
        function: Box::new(group.clone()),
        arguments,
    })
}
