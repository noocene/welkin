concat_base:
* ~as A     |->
Size ~as m   ->
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
bs |>
_ ||>
_ |>
elim < Size::add_zero_l_elim(m)
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