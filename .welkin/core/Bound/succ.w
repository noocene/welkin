succ:
Bound ->
Bound

bound |>
~match bound {
    new[nat](
        size,
        proof
    ) = Bound::new[
        Nat::succ(nat)
    ](
        Size::succ(size),
        Equal::map[
            'Nat,
            'Nat,
            > nat,
            Nat::from_size(size),
            nat |>
                nat < nat
                > Nat::succ(nat)
        ](proof)
    )
    : _ |> Bound
}