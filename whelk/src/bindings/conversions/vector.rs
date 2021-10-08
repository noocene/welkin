use crate::bindings::w;

impl<A> From<w::Vector<A>> for Vec<A> {
    fn from(mut vector: w::Vector<A>) -> Self {
        let mut vec = vec![];

        while let w::Vector::cons { head, tail } = vector {
            vector = *tail;
            vec.push(head);
        }

        vec
    }
}

impl<A> From<Vec<A>> for w::Vector<A> {
    fn from(vec: Vec<A>) -> Self {
        let mut vector = w::Vector::nil;

        for element in vec.into_iter().rev() {
            vector = w::Vector::cons {
                head: element,
                tail: Box::new(vector),
            };
        }

        vector
    }
}
