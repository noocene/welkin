use std::convert::TryInto;

use crate::bindings::w;

impl From<w::Char> for char {
    fn from(welkin: w::Char) -> Self {
        match welkin {
            w::Char::new { value } => {
                let bits: Vec<bool> = value.into();
                let bits: Vec<u8> = bits
                    .chunks_exact(8)
                    .rev()
                    .map(|a| {
                        let mut byte = 0u8;

                        for shift in 0..8 {
                            byte |= if a[shift] { 1 } else { 0 } << (7 - shift);
                        }

                        byte
                    })
                    .collect();

                char::from_u32(u32::from_be_bytes(bits.as_slice().try_into().unwrap())).unwrap()
            }
        }
    }
}

impl From<char> for w::Char {
    fn from(rust: char) -> Self {
        let mut word = vec![];

        for byte in (rust as u32).to_be_bytes() {
            for shift in 0..8u8 {
                if (1 << shift) & byte != 0 {
                    word.push(true)
                } else {
                    word.push(false)
                }
            }
        }

        w::Char::new { value: word.into() }
    }
}
