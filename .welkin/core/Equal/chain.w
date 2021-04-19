chain:
* ~as A |->
A ~as a |->
A ~as b |->
A ~as c |->
Equal[A, a, b] ->
Equal[A, b, c] ->
Equal[A, a, c]

A ||> a ||> _ ||> _ ||>
e |> d |>
~match e ~with b {
    refl = d
    : _ |> Equal[A, a, b]
}