use combine::{
    choice, optional, parser, parser::char::spaces, token as bare_token, value, Parser, Stream,
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use welkin_core::term::Show;

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbsolutePath(pub Vec<String>);

impl Debug for AbsolutePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = self.0.iter();

        if let Some(item) = items.next() {
            write!(f, "{}", item)?;

            for item in items {
                write!(f, "::{}", item)?;
            }
        }
        Ok(())
    }
}

impl Show for AbsolutePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <AbsolutePath as Debug>::fmt(self, f)
    }
}

pub mod term;
pub use term::Term;
pub mod util;

pub use bumpalo::Bump;
pub use util::{BumpBox, BumpString, BumpVec};

use term::{term, Context};
use util::{comma_separated, delimited, ident, string, token};

use self::util::{bare_ident, bump_many};

#[derive(Debug, Clone, PartialEq)]
pub struct Ident<'a>(pub BumpString<'a>);

impl<'a> Ident<'a> {
    pub fn from_str(a: &str, bump: &'a Bump) -> Self {
        Ident(BumpString::from_str(a, bump))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path<'a>(pub BumpVec<'a, Ident<'a>>);

#[derive(Debug, Clone)]
pub struct Variant<'a> {
    pub ident: Ident<'a>,
    pub inhabitants: BumpVec<'a, (Ident<'a>, Term<'a>, bool)>,
    pub indices: BumpVec<'a, Term<'a>>,
}

#[derive(Debug, Clone)]
pub struct Data<'a> {
    pub variants: BumpVec<'a, Variant<'a>>,
    pub type_arguments: BumpVec<'a, (Ident<'a>, Option<Term<'a>>, bool)>,
    pub indices: BumpVec<'a, (Ident<'a>, Term<'a>)>,
    pub ident: Ident<'a>,
}

#[derive(Debug, Clone)]
pub enum BlockItem<'a> {
    Data(Data<'a>),
}

#[derive(Debug, Clone)]
pub struct Declaration<'a> {
    pub ident: Ident<'a>,
    pub term: Term<'a>,
    pub ty: Term<'a>,
}

#[derive(Debug, Clone)]
pub enum Item<'a> {
    Block(BlockItem<'a>),
    Declaration(Declaration<'a>),
}

fn block_item_keyword<Input>() -> impl Parser<Input, Output = &'static str>
where
    Input: Stream<Token = char>,
{
    token('~').with(choice([string("data")]))
}

parser! {
    fn variant['a, Input](context: Context, bump: &'a Bump)(Input) -> Variant<'a>
    where
         [ Input: Stream<Token = char> ]
    {
        let bump = *bump;
        let context = &*context;
        (
            bare_ident(bump),
            optional(delimited('[', ']', comma_separated({
                let context = context.clone();
                move || (ident(bump).skip(token(':')), term(context.clone(), bump), value(true))
            }, bump))),
            optional(delimited('(', ')', comma_separated({
                let context = context.clone();
                move || (ident(bump).skip(token(':')), term(context.clone(), bump), value(false))
            }, bump))).skip(spaces()),
            optional(bare_token('~').and(string("with")).skip(spaces()).with(delimited('{','}', comma_separated(move || term(context.clone(), bump), bump)))).map(move |data| data.unwrap_or(BumpVec::new_in(bump)))
        )
            .map(move |(ident, erased_inhabitants, inhabitants, indices)| {
                let mut erased_inhabitants = erased_inhabitants.unwrap_or(BumpVec::new_in(bump));
                erased_inhabitants.append(&mut inhabitants.unwrap_or(BumpVec::new_in(bump)));
                Variant {
                    ident,
                    inhabitants: erased_inhabitants,
                    indices
                }
            })
    }
}

fn data<'a, Input>(
    ident: Ident<'a>,
    type_arguments: BumpVec<'a, (Ident<'a>, Option<Term<'a>>, bool)>,
    indices: BumpVec<'a, (Ident<'a>, Term<'a>)>,
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Data<'a>>
where
    Input: Stream<Token = char>,
{
    spaces()
        .with(comma_separated(
            move || variant(context.clone(), bump),
            bump,
        ))
        .map(move |variants| Data {
            variants,
            ident: ident.clone(),
            type_arguments: type_arguments.clone(),
            indices: indices.clone(),
        })
}

pub fn type_params<'a, Input>(
    bump: &'a Bump,
) -> impl Parser<Input, Output = BumpVec<'a, (Ident, Option<Term>, bool)>>
where
    Input: Stream<Token = char>,
{
    bump_many(
        move || {
            (bare_ident(bump).skip(spaces()), value(None), value(true))
                .or(delimited(
                    '[',
                    ']',
                    ident(bump)
                        .skip(token(':'))
                        .and(term(Default::default(), bump)),
                )
                .skip(spaces())
                .map(|(ident, term)| (ident, Some(term), true)))
                .or(delimited(
                    '(',
                    ')',
                    (
                        ident(bump),
                        optional(token(':').with(term(Default::default(), bump))),
                    ),
                )
                .skip(spaces())
                .map(|(ident, term)| (ident, term, false)))
        },
        bump,
    )
}

parser! {
    fn block_item['a, Input](context: Context, bump: &'a Bump)(Input) -> BlockItem<'a>
    where
         [ Input: Stream<Token = char> ]
    {
        let bump = *bump;
        let context = context.clone();
        block_item_keyword().then(move |kw| {
            let context = context.clone();
            match kw {
                "data" => (
                        ident(bump).skip(spaces()),
                        type_params(bump),
                        optional(
                            token('~')
                                .and(string("with"))
                                .skip(spaces())
                                .with(delimited('{','}', comma_separated({
                                    let context = context.clone();
                                    move || (ident(bump).skip(token(':')), term(context.clone(), bump))
                                }, bump)).skip(spaces()))
                        )
                    ).then(move |(ident, type_arguments, indices)| {
                    delimited(
                        '{',
                        '}',
                        data(ident, type_arguments, indices.unwrap_or(BumpVec::new_in(bump)), context.clone(), bump).map(BlockItem::Data)
                    )
                }),
                _ => panic!()
            }
        })
    }
}

pub fn declaration<'a, Input>(
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Declaration<'a>>
where
    Input: Stream<Token = char>,
{
    (
        ident(bump).skip(token(':')),
        term(context.clone(), bump),
        term(context, bump),
    )
        .map(|(ident, ty, term)| Declaration { ident, ty, term })
}

pub fn item<'a, Input>(bump: &'a Bump) -> impl Parser<Input, Output = Item<'a>>
where
    Input: Stream<Token = char>,
{
    let parser = block_item(Default::default(), bump).map(Item::Block);
    let parser = parser.or(declaration(Default::default(), bump).map(Item::Declaration));
    parser
}

pub fn items<'a, Input>(bump: &'a Bump) -> impl Parser<Input, Output = BumpVec<'a, Item>>
where
    Input: Stream<Token = char>,
{
    bump_many(move || item(bump), bump)
}
