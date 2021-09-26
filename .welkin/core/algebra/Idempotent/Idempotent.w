~data Idempotent A [F: A -> A] {
    new(proof: A ~as a -> Equal[A, F(a), F(F(a))])
}