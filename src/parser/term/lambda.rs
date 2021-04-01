use combine::{Parser, Stream};

use crate::parser::Ident;

use super::{term, Context, Term};

pub fn lambda<Input>(ident: Ident, context: Context) -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    term(context.clone())
        .map(Box::new)
        .map(move |body| Term::Lambda {
            argument: ident.clone(),
            erased: false,
            body,
        })
}
