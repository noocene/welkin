~data LeftIdentity A [a: A] [F: A -> A -> A] {
    new(proof: A ~as b -> Equal[A, F(a, b), b])
}