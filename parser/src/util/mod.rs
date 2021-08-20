pub use combine::token as bare_token;
use combine::{
    between, optional, parser,
    parser::char::{letter, spaces, string as bare_string},
    value, Parser, Stream,
};
use std::mem::replace;

use bumpalo::Bump;

mod bump;
pub use bump::{BumpBox, BumpString, BumpVec};

use super::{Ident, Path};

pub fn bump_many<'a, Input, T: 'a, P: Parser<Input, Output = T>>(
    p: impl Fn() -> P,
    bump: &'a Bump,
) -> impl Parser<Input, Output = BumpVec<'a, T>>
where
    Input: Stream<Token = char>,
{
    let mut buffer = BumpVec::new_in(bump);
    parser(move |input| {
        buffer.clear();
        let mut iter = p().iter(input);
        buffer.extend(&mut iter);
        iter.into_result(replace(&mut buffer, BumpVec::new_in(bump)))
    })
}

pub fn bump_many1<'a, Input, T: Clone, P: Parser<Input, Output = T>>(
    p: impl Fn() -> P + Clone,
    bump: &'a Bump,
) -> impl Parser<Input, Output = BumpVec<'a, T>>
where
    Input: Stream<Token = char>,
{
    p().then(move |first| {
        let mut buffer = BumpVec::new_in(bump);
        let p = p.clone();
        parser(move |input| {
            buffer.clear();
            buffer.extend(Some(first.clone()));
            let mut iter = p().iter(input);
            buffer.extend(&mut iter);
            iter.into_result(replace(&mut buffer, BumpVec::new_in(bump)))
        })
    })
}

pub fn bump_string<'a, Input>(bump: &'a Bump) -> impl Parser<Input, Output = BumpString<'a>>
where
    Input: Stream<Token = char>,
{
    letter().or(bare_token('_')).then(move |first| {
        let mut buffer = BumpString::new_in(bump);
        parser(move |input| {
            buffer.clear();
            buffer.extend(Some(first));
            let mut iter = letter().or(bare_token('_')).iter(input);
            buffer.extend(&mut iter);
            iter.into_result(replace(&mut buffer, BumpString::new_in(bump)))
        })
    })
}

pub fn bare_ident<'a, Input>(bump: &'a Bump) -> impl Parser<Input, Output = Ident<'a>>
where
    Input: Stream<Token = char>,
{
    bump_string(bump).map(Ident)
}

pub fn bare_path<'a, Input>(bump: &'a Bump) -> impl Parser<Input, Output = Path<'a>>
where
    Input: Stream<Token = char>,
{
    bump_many1(
        move || bare_ident(bump).skip(optional(bare_string("::"))),
        bump,
    )
    .map(Path)
}

pub fn ident<'a, Input>(bump: &'a Bump) -> impl Parser<Input, Output = Ident<'a>>
where
    Input: Stream<Token = char>,
{
    spaces().with(bare_ident(bump))
}

pub fn token<Input>(c: char) -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
{
    spaces().with(bare_token(c))
}

pub fn string<Input>(s: &'static str) -> impl Parser<Input, Output = &'static str>
where
    Input: Stream<Token = char>,
{
    spaces().with(bare_string(s))
}

pub fn comma_separated<'a, Input, T: Clone + 'a, P: Parser<Input, Output = T>>(
    p: impl Fn() -> P + Clone,
    bump: &'a Bump,
) -> impl Parser<Input, Output = BumpVec<'a, T>>
where
    Input: Stream<Token = char>,
{
    p().then(move |first| {
        let mut buffer = BumpVec::new_in(bump);
        let p = p.clone();
        parser(move |input| {
            buffer.clear();
            buffer.extend(Some(first.clone()));
            let mut iter = bare_token(',').skip(spaces()).with(p()).iter(input);
            buffer.extend(&mut iter);
            iter.into_result(replace(&mut buffer, BumpVec::new_in(bump)))
        })
    })
    .or(value(BumpVec::new_in(bump)))
}

pub fn comma_separated1<'a, Input, T: Clone + 'a, P: Parser<Input, Output = T>>(
    p: impl Fn() -> P + Clone,
    bump: &'a Bump,
) -> impl Parser<Input, Output = BumpVec<'a, T>>
where
    Input: Stream<Token = char>,
{
    p().then(move |first| {
        let mut buffer = BumpVec::new_in(bump);
        let p = p.clone();
        parser(move |input| {
            buffer.clear();
            buffer.extend(Some(first.clone()));
            let mut iter = bare_token(',').skip(spaces()).with(p()).iter(input);
            buffer.extend(&mut iter);
            iter.into_result(replace(&mut buffer, BumpVec::new_in(bump)))
        })
    })
}

pub fn delimited<Input, T>(
    a: char,
    b: char,
    parser: impl Parser<Input, Output = T>,
) -> impl Parser<Input, Output = T>
where
    Input: Stream<Token = char>,
{
    between(bare_token(a), token(b), parser)
}
