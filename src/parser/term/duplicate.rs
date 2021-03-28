use combine::{parser::char::spaces, Parser, Stream};

use crate::parser::Ident;

use super::{term, Context, Term};

pub fn duplicate<Input>(binding: Ident, context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    (
        term(context.clone()).map(Box::new).skip(spaces()),
        term(context.clone()).map(Box::new),
    )
        .map(move |(expression, body)| Term::Duplicate {
            expression,
            binding: binding.clone(),
            body,
        })
}
