use welkin_binding::{check_all_in, concrete_type, Adt, Definitions, FromWelkin, ToWelkin};

#[derive(Debug, Adt, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum Bool {
    r#true,
    r#false,
}

#[derive(Debug, Adt, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum Pair<A, B> {
    new { left: A, right: B },
}

#[derive(Debug, Adt)]
#[allow(non_camel_case_types)]
pub enum Unit {
    new,
}

#[derive(Debug, Adt, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum Vector<A> {
    nil,
    cons {
        head: A,
        #[inductive]
        tail: Box<Vector<A>>,
    },
}

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

    let defs = std::fs::read_to_string("../../whelk/welkin/defs.mwc").unwrap();

    let defs: Definitions = defs.parse().unwrap();

    check_all_in::<Pair<Unit, Bool>>(&defs).unwrap();
}
