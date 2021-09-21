use welkin_core::term::{alloc::IntoInner, Index, Term};

use crate::interface::{FromWelkin, WSized, WString};

#[derive(Debug, Clone)]
pub enum Request {
    Prompt,
    Print(WSized<WString>),
}

#[derive(Debug)]
pub enum InvalidRequest {
    InvalidRequest,
    InvalidString(<WSized<WString> as FromWelkin>::Error),
}

impl FromWelkin for Request {
    type Error = InvalidRequest;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Lambda { body, .. } = body.into_inner() {
                if let Term::Apply {
                    argument, function, ..
                } = body.into_inner()
                {
                    match function.into_inner() {
                        Term::Variable(Index(0)) => return Ok(Request::Prompt),
                        Term::Variable(Index(1)) => {
                            return Ok(Request::Print(
                                FromWelkin::from_welkin(argument.into_inner())
                                    .map_err(InvalidRequest::InvalidString)?,
                            ))
                        }
                        _ => return Err(InvalidRequest::InvalidRequest),
                    }
                }
            }
        }
        Err(InvalidRequest::InvalidRequest)
    }
}
