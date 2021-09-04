from_size:
Size ->
Bound

n |>
Bound::new[
    nat < Nat::from_size(n)
    nat
](
    n,
    Equal::refl[
        'Nat, 
        Nat::from_size(n)
    ]
)