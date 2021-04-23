~data RightIdentity A B [a: A] [F: B -> A -> B] {
    new(proof: B ~as b -> Equal[A, F(b, a), b])
}