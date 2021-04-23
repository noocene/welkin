magma:
* ~as A     |->
Semigroup[A] ->
Magma[A]

A ||>
semigroup |>
~match semigroup {
    new(magma, _) = magma
    : _ |> Magma[A]
}