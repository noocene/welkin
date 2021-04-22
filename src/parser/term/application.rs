use std::{cell::RefCell, rc::Rc};

use bumpalo::Bump;
use combine::{any, look_ahead, optional, parser::combinator::Either, value, Parser, Stream};

use crate::parser::{
    util::{comma_separated1, delimited, BumpBox},
    BumpVec,
};

use super::{term, Context, Term};

pub fn concrete_application<'a, Input>(
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = BumpVec<'a, Term<'a>>>
where
    Input: Stream<Token = char>,
{
    delimited(
        '(',
        ')',
        comma_separated1(
            {
                let context = context.clone();
                move || term(context.clone(), bump)
            },
            bump,
        ),
    )
}

pub fn application<'a, Input>(
    erased: bool,
    group: Rc<RefCell<Option<Term<'a>>>>,
    context: Context,
    bump: &'a Bump,
) -> impl Parser<Input, Output = Term<'a>>
where
    Input: Stream<Token = char>,
{
    if erased {
        let parser = delimited(
            '[',
            ']',
            comma_separated1(
                {
                    let context = context.clone();
                    move || term(context.clone(), bump)
                },
                bump,
            ),
        )
        .map(move |arguments| Term::Application {
            erased: true,
            function: BumpBox::new_in(group.borrow_mut().take().unwrap(), bump),
            arguments,
        });
        Either::Left(parser.then(move |term| {
            let term = Rc::new(RefCell::new(Some(term)));
            look_ahead(optional(any())).then({
                let context = context.clone();
                move |token| {
                    if token == Some('(') {
                        Either::Left(concrete_application(context.clone(), bump).map({
                            let term = term.clone();
                            move |arguments| Term::Application {
                                erased: false,
                                function: BumpBox::new_in(term.borrow_mut().take().unwrap(), bump),
                                arguments,
                            }
                        }))
                    } else {
                        Either::Right(value(term.borrow_mut().take().unwrap()))
                    }
                }
            })
        }))
    } else {
        Either::Right(
            concrete_application(context, bump).map(move |arguments| Term::Application {
                erased: false,
                function: BumpBox::new_in(group.borrow_mut().take().unwrap(), bump),
                arguments,
            }),
        )
    }
}
