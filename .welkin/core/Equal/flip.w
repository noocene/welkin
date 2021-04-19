flip:
* ~as A |->
A ~as a |->
A ~as b |->
Equal[
    A, a, b
]        ->
Equal[
    A, b, a
]

A ||> a ||> _ ||>
e |>
~match e ~with b {
    refl = Equal::refl[A, a]
    : _ |> Equal[A, b, a]
}