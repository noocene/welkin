use combine::{
    between, many1,
    parser::char::{letter, spaces, string as bare_string},
    sep_by, sep_by1, token as bare_token, Parser, Stream,
};

use super::{Ident, Path};

pub fn bare_ident<Input>() -> impl Parser<Input, Output = Ident>
where
    Input: Stream<Token = char>,
{
    many1(letter().or(bare_token('_'))).map(Ident)
}

pub fn bare_path<Input>() -> impl Parser<Input, Output = Path>
where
    Input: Stream<Token = char>,
{
    sep_by1(bare_ident(), bare_string("::")).map(Path)
}

pub fn ident<Input>() -> impl Parser<Input, Output = Ident>
where
    Input: Stream<Token = char>,
{
    spaces().with(bare_ident())
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

pub fn comma_separated<Input, T>(
    parser: impl Parser<Input, Output = T>,
) -> impl Parser<Input, Output = Vec<T>>
where
    Input: Stream<Token = char>,
{
    sep_by(parser, bare_token(','))
}

pub fn comma_separated1<Input, T>(
    parser: impl Parser<Input, Output = T>,
) -> impl Parser<Input, Output = Vec<T>>
where
    Input: Stream<Token = char>,
{
    sep_by1(parser, bare_token(','))
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
