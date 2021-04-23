~data Monoid A {
    new(
        semigroup: Semigroup[A],
        identity: Sigma[
            A,
            a |> Identity[
                A,
                a,
                Magma::operation[A](
                    Semigroup::magma[A](semigroup)
                )
            ]
        ]
    )
}