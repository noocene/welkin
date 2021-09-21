use welkin_core::term::{alloc::IntoInner, Index, Term};

use super::{word::InvalidWord, FromWelkin, ToWelkin, Word};

#[derive(Clone, Debug)]
pub struct Char(pub char);

#[derive(Debug)]
pub enum InvalidChar {
    InvalidChar,
    InvalidWord(InvalidWord),
}

impl From<InvalidWord> for InvalidChar {
    fn from(e: InvalidWord) -> Self {
        InvalidChar::InvalidWord(e)
    }
}

impl FromWelkin for Char {
    type Error = InvalidChar;

    fn from_welkin(term: welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
        if let Term::Lambda { body, .. } = term {
            if let Term::Apply { argument, .. } = body.into_inner() {
                let bits = Word::from_welkin(argument.into_inner())?.0;
                let mut bytes = [0u8; 4];

                for (bit, bits) in bits.as_slice().chunks(8).rev().enumerate() {
                    let mut byte = 0u8;

                    for idx in bits
                        .iter()
                        .enumerate()
                        .filter(|(_, bit)| **bit)
                        .map(|(idx, _)| idx)
                    {
                        byte |= 1 << (7 - idx);
                    }

                    bytes[bit] = byte;
                }

                Ok(Char(
                    char::from_u32(u32::from_be_bytes(bytes)).ok_or(InvalidChar::InvalidChar)?,
                ))
            } else {
                Err(InvalidChar::InvalidChar)
            }
        } else {
            Err(InvalidChar::InvalidChar)
        }
    }
}

impl ToWelkin for Char {
    type Error = InvalidWord;

    fn to_welkin(self) -> Result<Term<String>, Self::Error> {
        let character = (self.0 as u32).to_be_bytes();
        let mut bits = vec![];
        for byte in character {
            for bit in 0..8u8 {
                if ((1 << bit) & byte) != 0 {
                    bits.push(true);
                } else {
                    bits.push(false);
                }
            }
        }

        let word: Word = Word(bits);

        let mut term = word.to_welkin()?;

        term = Term::Apply {
            erased: false,
            argument: Box::new(term),
            function: Box::new(Term::Variable(Index(0))),
        };
        term = Term::Lambda {
            erased: false,
            body: Box::new(term),
        };

        Ok(term)
    }
}
