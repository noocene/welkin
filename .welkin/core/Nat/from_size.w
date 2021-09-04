from_size:
Size ->
'Nat

n |>
Size::fold[
	Nat
](
    n,
    > Nat::zero,
    > n |> Nat::succ(n)
)