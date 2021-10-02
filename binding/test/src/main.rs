use welkin_binding::{Adt, FromWelkin, ToWelkin};

#[derive(Debug, Adt)]
#[allow(non_camel_case_types)]
pub enum Bool {
    r#true,
    r#false,
}

#[derive(Debug, Adt)]
#[allow(non_camel_case_types)]
pub enum Pair<A, B> {
    new { left: A, right: B },
}

fn main() {
    let pair = Pair::new {
        left: Pair::new {
            left: Bool::r#true,
            right: Bool::r#false,
        },
        right: Bool::r#false,
    };

    println!(
        "{:?}",
        Pair::<Pair<Bool, Bool>, Bool>::from_welkin(pair.to_welkin().unwrap()).unwrap()
    );
}
