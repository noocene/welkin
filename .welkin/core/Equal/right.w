right:
* ~as A |->
A ~as a |->
A ~as b |->
A ~as c |->
Equal[
    A, c, a
]        ->
Equal[
    A, c, b
]        ->
Equal[
    A, a, b
]

A ||> a ||> b ||> c ||>
e |> d |>
Equal::chain[A, a, c, b](
    e, Equal::flip[A, c, a](d)
)