map:
* ~as A |->
* ~as B |->
A ~as a |->
A ~as b |->
(
    A -> B
) ~as f |->
Equal[
    A, a, b
]        ->
Equal[
    B, f(a), f(b)
]

_ ||> B ||> a ||> _ ||> f ||>
e |>
~match e ~with b {
    refl = Equal::refl[B, f(a)]
    : _ |> Equal[B, f(a), f(b)]
}