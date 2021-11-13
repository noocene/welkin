out:
* ~as A |->
Rec[A] ->
(
    Rec[A] |->
    A
)

A ||>
rec |>
~match rec {
    in(out) = out
    : _ |> Rec[A] |-> A
}