~data RightIdentity A B [a: A] [F: B -> A -> B] {
    new(proof: B ~as b -> Equal[B, F(b, a), b])
}