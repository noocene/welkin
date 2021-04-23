~data LeftIdentity A B [a: A] [F: A -> B -> B] {
    new(proof: B ~as b -> Equal[A, F(a, b), b])
}