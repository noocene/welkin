fold:
* ~as A         |->
* ~as B         |->
Size ~as size    ->
'B               ->
'Vector[
    A, size
] ->
'(
    B ->
    A ->
    B
) ->
'B

A ||>
B ||>
n |>
initial |>
vector |>
call |>
initial < initial
vector < vector
call < call
Size::recurse[
    Unit,
    n |> _ |> Pair::new[*, *](
        Pair[B, Vector[A, n]],
        B
    ),
    Unit::new
](
    n,
    > Pair::new[B, Vector[A, n]](initial, vector),
    > Vector::fold_base[A, B],
    > Vector::fold_cont[A, B](call)
)
