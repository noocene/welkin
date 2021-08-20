left:
* ~as A   |->
* ~as B   |->
Pair[A, B] ->
A

A ||> _ ||>
pair |>
~match pair {
    new(left, right) = left
    : _ |> A
}