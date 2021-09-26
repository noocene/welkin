~data LeftIdentity A B [a: A] [F: A -> B -> B] {
    new(proof: B ~as b -> Equal[B, F(a, b), b])
}