use bumpalo::Bump;
use combine::{Parser, Stream};

use crate::{util::BumpBox, Ident};

use super::{term, Context, Term};

pub fn lambda<'a, Input>(
    erased: bool,
    ident: Ident<'a>,
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Term<'a>>
where
    Input: Stream<Token = char>,
{
    term(context.clone(), bump)
        .map(move |a| BumpBox::new_in(a, bump))
        .map(move |body| Term::Lambda {
            argument: ident.clone(),
            erased,
            body,
        })
}
