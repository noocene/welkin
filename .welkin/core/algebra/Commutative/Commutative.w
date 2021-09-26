~data Commutative A [F: A -> A -> A] {
    new(proof:
        A ~as a ->
        A ~as b ->
        Equal[A, F(a, b), F(b, a)]
    )
}