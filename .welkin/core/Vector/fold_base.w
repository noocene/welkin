fold_base:
* ~as A |->
* ~as B |->
Unit    |->
Pair[B, Vector[
    A, Size::zero
]]       ->
B

A ||>
B ||>
_ ||>
pair |>
~match pair {
    new(accumulator, _) = accumulator
    : _ |> B
}