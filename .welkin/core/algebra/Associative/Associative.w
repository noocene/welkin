~data Associative A [F: A -> A -> A] {
    new(proof:
        A ~as a ->
        A ~as b ->
        A ~as c ->
        Equal[A, F(F(a, b), c), F(a, F(b, c))]
    )
}