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
Size::recurse_indexed[
    Unit,
    n |> _ |> Pair::new[*, *](
        Vector[A, n],
        Vector[A, Size::add(n, m)]
    ),
    Unit::new
](
    n,
    as,
    > Vector::concat_base[A](m, bs),
    > Vector::concat_cont[A, m]
)
