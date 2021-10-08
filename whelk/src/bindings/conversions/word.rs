use crate::bindings::w;

impl From<w::Word> for Vec<bool> {
    fn from(mut word: w::Word) -> Self {
        let mut vector = vec![];

        while let Some(element) = match word {
            w::Word::empty => None,
            w::Word::low { r#after } => {
                word = *after;
                Some(false)
            }
            w::Word::high { r#after } => {
                word = *after;
                Some(true)
            }
        } {
            vector.push(element);
        }

        vector
    }
}

impl From<Vec<bool>> for w::Word {
    fn from(vector: Vec<bool>) -> Self {
        let mut word = w::Word::empty;

        for element in vector.into_iter() {
            if element {
                word = w::Word::high {
                    after: Box::new(word),
                };
            } else {
                word = w::Word::low {
                    after: Box::new(word),
                };
            }
        }

        word
    }
}
