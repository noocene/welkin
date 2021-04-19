rewrite:
* ~as A       |->
A ~as a       |->
A ~as b       |->
(
    A -> *
) ~as prop    |->
Equal[A, a, b] ->
prop(a)        ->
prop(b)

_ ||> _ ||> _ ||> prop ||>
e |> a |>
~match e ~with b {
    refl = a
    : _ |> prop(b)
}