concat:
* ~as A     |->
Semigroup[A] ->
A -> A -> A

A ||>
semigroup |>
~match semigroup {
    new(concat) = concat
    : _ |> A -> A -> A
}