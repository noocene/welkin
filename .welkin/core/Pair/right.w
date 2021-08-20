left:
* ~as A   |->
* ~as B   |->
Pair[A, B] ->
B

_ ||> B ||>
pair |>
~match pair {
    new(left, right) = right
    : _ |> B
}