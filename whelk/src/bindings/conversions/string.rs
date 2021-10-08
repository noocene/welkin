use crate::bindings::w;

impl From<w::String> for String {
    fn from(string: w::String) -> Self {
        match string {
            w::String::new { value } => Vec::from(value)
                .into_iter()
                .map(|a| char::from(a))
                .collect(),
        }
    }
}

impl From<String> for w::String {
    fn from(string: String) -> Self {
        w::String::new {
            value: string
                .chars()
                .into_iter()
                .map(|a| w::Char::from(a))
                .collect::<Vec<_>>()
                .into(),
        }
    }
}
