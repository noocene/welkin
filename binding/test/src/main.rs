use welkin_binding::{bind, canonically_equivalent_all_in, concrete_type, FromWelkin, ToWelkin};

impl<A> From<Vec<A>> for Vector<A> {
    fn from(vec: Vec<A>) -> Self {
        let mut vector = Vector::nil;
        for element in vec.into_iter().rev() {
            vector = Vector::cons {
                head: element,
                tail: Box::new(vector),
            }
        }
        vector
    }
}

bind! {
    #[path = "../../whelk/welkin/defs"]
    #[include(
        Unit,
        Bool,
        Pair,
        Vector,
        Word,
        Char,
        String,
    )]
    pub mod w {}
}

use w::*;

fn main() {
    let pair = Pair::new {
        left: Pair::new {
            left: Bool::r#true,
            right: Bool::r#false,
        },
        right: Bool::r#false,
    };

    assert_eq!(
        Pair::<Pair<Bool, Bool>, Bool>::from_welkin(pair.clone().to_welkin().unwrap()).unwrap(),
        pair
    );

    let vector = Vector::<Bool>::from(vec![Bool::r#true, Bool::r#false]);
    assert_eq!(
        Vector::<Bool>::from_welkin(vector.clone().to_welkin().unwrap()).unwrap(),
        vector
    );

    type Test = Vector<Pair<Bool, Vector<Bool>>>;

    println!("{:?}", concrete_type::<Test>());

    let defs = std::fs::read("../../whelk/welkin/defs").unwrap();

    canonically_equivalent_all_in::<Test>(&defs).unwrap();
}
