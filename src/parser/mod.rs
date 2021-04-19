use combine::{
    attempt, choice, many, many1, optional, parser,
    parser::char::{letter, spaces},
    token as bare_token, value, Parser, Stream,
};

pub mod term;
pub use term::Term;
mod util;

use term::{term, Context};
use util::{comma_separated, delimited, ident, string, token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(pub Vec<Ident>);

#[derive(Debug, Clone)]
pub struct Variant {
    pub ident: Ident,
    pub inhabitants: Vec<(Ident, Term, bool)>,
    pub indices: Vec<Term>,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub variants: Vec<Variant>,
    pub type_arguments: Vec<(Ident, Option<Term>, bool)>,
    pub indices: Vec<(Ident, Term)>,
    pub ident: Ident,
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Data(Data),
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub ident: Ident,
    pub term: Term,
    pub ty: Term,
}

#[derive(Debug, Clone)]
pub enum Item {
    Block(BlockItem),
    Declaration(Declaration),
}

fn block_item_keyword<Input>() -> impl Parser<Input, Output = &'static str>
where
    Input: Stream<Token = char>,
{
    token('~').with(choice([string("data")]))
}

parser! {
    fn variant[Input](context: Context)(Input) -> Variant
    where
         [ Input: Stream<Token = char> ]
    {
        (
            ident(),
            optional(delimited('[', ']', comma_separated((ident().skip(token(':')), term(context.clone()), value(true))))),
            optional(delimited('(', ')', comma_separated((ident().skip(token(':')), term(context.clone()), value(false))))),
            optional(attempt(token('~').and(string("with")).skip(spaces()).with(delimited('{','}', comma_separated(term(context.clone())))))).map(|data| data.unwrap_or(vec![]))
        )
            .map(|(ident, erased_inhabitants, inhabitants, indices)| {
                let mut erased_inhabitants = erased_inhabitants.unwrap_or(vec![]);
                erased_inhabitants.append(&mut inhabitants.unwrap_or(vec![]));
                Variant {
                    ident,
                    inhabitants: erased_inhabitants,
                    indices
                }
            })
    }
}

fn data<Input>(
    ident: Ident,
    type_arguments: Vec<(Ident, Option<Term>, bool)>,
    indices: Vec<(Ident, Term)>,
    context: Context,
) -> impl Parser<Input, Output = Data>
where
    Input: Stream<Token = char>,
{
    comma_separated(variant(context.clone())).map(move |variants| Data {
        variants,
        ident: ident.clone(),
        type_arguments: type_arguments.clone(),
        indices: indices.clone(),
    })
}

pub fn type_params<Input>() -> impl Parser<Input, Output = Vec<(Ident, Option<Term>, bool)>>
where
    Input: Stream<Token = char>,
{
    many(
        (
            many1(letter().or(bare_token('_')))
                .skip(spaces())
                .map(Ident),
            value(None),
            value(true),
        )
            .or(delimited(
                '[',
                ']',
                ident().skip(token(':')).and(term(Default::default())),
            )
            .skip(spaces())
            .map(|(ident, term)| (ident, Some(term), true)))
            .or(delimited(
                '(',
                ')',
                (
                    ident(),
                    optional(attempt(token(':').with(term(Default::default())))),
                ),
            )
            .skip(spaces())
            .map(|(ident, term)| (ident, term, false))),
    )
}

parser! {
    fn block_item[Input](context: Context)(Input) -> BlockItem
    where
         [ Input: Stream<Token = char> ]
    {
        attempt(block_item_keyword()).then(|kw| {
            let context = context.clone();
            match kw {
                "data" => (
                        ident().skip(spaces()),
                        type_params(),
                        optional(
                            attempt(token('~')
                                .and(string("with"))
                                .skip(spaces())
                                .with(delimited('{','}', comma_separated((ident().skip(token(':')), term(context.clone())))).skip(spaces())))
                        )
                    ).then(move |(ident, type_arguments, indices)| {
                    delimited(
                        '{',
                        '}',
                        data(ident, type_arguments, indices.unwrap_or(vec![]), context.clone()).map(BlockItem::Data)
                    )
                }),
                _ => panic!()
            }
        })
    }
}

pub fn declaration<Input>(context: Context) -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
{
    (
        ident().skip(token(':')),
        term(context.clone()),
        term(context),
    )
        .map(|(ident, ty, term)| Declaration { ident, ty, term })
}

pub fn item<Input>() -> impl Parser<Input, Output = Item>
where
    Input: Stream<Token = char>,
{
    let parser = block_item(Default::default()).map(Item::Block);
    let parser = parser.or(declaration(Default::default()).map(Item::Declaration));
    parser
}

pub fn items<Input>() -> impl Parser<Input, Output = Vec<Item>>
where
    Input: Stream<Token = char>,
{
    many(item())
}
