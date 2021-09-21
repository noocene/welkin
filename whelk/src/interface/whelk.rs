#[allow(unused_imports)]
use welkin_core::{
    net::{Net, NetError, VisitNetExt},
    term::{
        alloc::{IntoInner, System},
        DefinitionResult, Definitions, None, StratificationError, Term,
    },
};

use crate::interface::WString;

use super::{
    box_poly::InvalidBoxPoly,
    sized::{InvalidSized, SizedToWelkinError},
    BoxPoly, FromWelkin, ToWelkin, WSized,
};

#[derive(Clone, Debug)]
pub struct Whelk(Term<String>);

impl FromWelkin for Whelk {
    type Error = WhelkError;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Apply { argument, .. } = body.into_inner() {
                return Ok(Whelk(argument.into_inner()));
            }
        }
        Err(WhelkError)
    }
}

#[derive(Debug)]
pub struct WhelkError;

#[derive(Debug)]
pub enum WhelkCallError {
    InvalidOutput(InvalidBoxPoly<WSized<WString>>),
    InvalidInput(SizedToWelkinError<WString>),
    Net(NetError<String, None, System>),
    Stratification(StratificationError<String>),
}

#[derive(Clone)]
pub struct NullDefinitions;

impl Definitions<String> for NullDefinitions {
    fn get(&self, _: &String) -> Option<DefinitionResult<Term<String>>> {
        None
    }
}

impl Whelk {
    pub fn call(&self, input: String) -> Result<String, WhelkCallError> {
        let mut main = Term::Apply {
            erased: false,
            function: Box::new(self.0.clone()),
            argument: Box::new(
                BoxPoly(WSized(WString(input)))
                    .to_welkin()
                    .map_err(WhelkCallError::InvalidInput)?,
            ),
        };

        // let main = {
        //     let mut net = main
        //         .stratified(&NullDefinitions)
        //         .map_err(WhelkCallError::Stratification)?
        //         .into_net::<Net<u32>>()
        //         .map_err(WhelkCallError::Net)?;
        //     net.reduce_all();
        //     net.read_term(welkin_core::net::Index(0))
        // };

        main.normalize(&NullDefinitions);

        BoxPoly::<WSized<WString>>::from_welkin(main)
            .map_err(WhelkCallError::InvalidOutput)
            .map(|a| ((a.0).0).0)
    }
}
