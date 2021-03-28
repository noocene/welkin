use combine::{choice, many, optional, parser, Parser, Stream};

pub mod term;
pub use term::Term;
mod util;

use term::{term, Context};
use util::{comma_separated, delimited, ident, ident_list, string, token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

#[derive(Debug, Clone)]
pub struct Path(pub Vec<Ident>);

#[derive(Debug, Clone)]
pub struct Variant {
    pub ident: Ident,
    pub inhabitants: Vec<Term>,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub variants: Vec<Variant>,
    pub type_arguments: Vec<Ident>,
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
            optional(delimited('(', ')', comma_separated(term(context.clone())))),
        )
            .map(|(ident, inhabitants)| Variant {
                ident,
                inhabitants: inhabitants.unwrap_or(vec![]),
            })
    }
}

fn data<Input>(
    ident: Ident,
    type_arguments: Vec<Ident>,
    context: Context,
) -> impl Parser<Input, Output = Data>
where
    Input: Stream<Token = char>,
{
    comma_separated(variant(context.clone())).map(move |variants| Data {
        variants,
        ident: ident.clone(),
        type_arguments: type_arguments.clone(),
    })
}

parser! {
    fn block_item[Input](context: Context)(Input) -> BlockItem
    where
         [ Input: Stream<Token = char> ]
    {
        block_item_keyword().then(|kw| {
            let context = context.clone();
            match kw {
                "data" => (ident(), ident_list()).then(move |(ident, type_arguments)| {
                    delimited(
                        '{',
                        '}',
                        data(ident, type_arguments, context.clone()).map(BlockItem::Data)
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
        term(context.clone()).skip(token('=')),
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
