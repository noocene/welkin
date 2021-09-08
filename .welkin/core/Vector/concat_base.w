concat_base:
* ~as A     |->
Size ~as m   ->
Equal[
    Size,
    Size::add(Size::zero, m),
    m
]            ->
Vector[A, m] ->
Unit        |->
Vector[
    A, Size::zero
]            ->
Vector[
    A, Size::add(Size::zero, m)
]

A ||>
m |>
elim |>
bs |>
_ ||>
_ |>
Equal::rewrite[
    Size,
    m,
    Size::add(Size::zero, m),
    n |> Vector[A, n]
](Equal::flip[
    Size,
    Size::add(Size::zero, m),
    m
](elim), bs)