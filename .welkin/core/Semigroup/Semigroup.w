~data Semigroup A {
    new(
        operation: Magma[A],
        associative: Associative[A, Magma::operation[A](operation)]
    )
}