concat:
* ~as A     |->
Size ~as n   ->
Size ~as m   ->
'Vector[A, n] ->
'Vector[A, m] ->
'Vector[A, Size::add(n, m)]

A ||>
n |>
m |>
as |>
bs |>
bs < bs
elim < Size::add_zero_l_elim(m)
Size::recurse[
    Unit,
    n |> _ |> Pair::new[*, *](
        Vector[A, n],
        Vector[A, Size::add(n, m)]
    ),
    Unit::new
](
    n,
    as,
    > Vector::concat_base[A, m](elim, bs),
    > Vector::concat_cont[A, m]
)
