left:
* ~as A |->
A ~as a |->
A ~as b |->
A ~as c |->
Equal[
    A, a, c
]        ->
Equal[
    A, b, c
]        ->
Equal[
    A, a, b
]

A ||> a ||> b ||> c ||>
e |> d |>
Equal::chain[A, a, c, b](
    e, Equal::flip[A, b, c](d)
)