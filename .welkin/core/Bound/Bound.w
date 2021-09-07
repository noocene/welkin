~data Bound {
    new[nat: Nat](
        size: Size,
        proof: Equal[
            'Nat,
            > nat,
            Nat::from_size(size)
        ]
    )
}