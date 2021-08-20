use bumpalo::Bump;
use combine::{parser::char::spaces, Parser, Stream};

use crate::{util::BumpBox, Ident};

use super::{term, Context, Term};

pub fn duplicate<'a, Input>(
    binding: Ident<'a>,
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Term<'a>>
where
    Input: Stream<Token = char>,
{
    (
        term(context.clone(), bump)
            .map(move |a| BumpBox::new_in(a, bump))
            .skip(spaces()),
        term(context.clone(), bump).map(move |a| BumpBox::new_in(a, bump)),
    )
        .map(move |(expression, body)| Term::Duplicate {
            expression,
            binding: binding.clone(),
            body,
        })
}
