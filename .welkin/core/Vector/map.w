map:
* ~as A         |->
* ~as B         |->
Size ~as size    ->
'Vector[A, size] ->
'(A -> B)        ->
'Vector[B, size]

A ||>
B ||>
size |>
vector |>
call |>
call < call
Size::recurse_indexed[
    Unit,
    n |> _ |> Pair::new[*, *](
        Vector[A, n],
        Vector[B, n]
    ),
    Unit::new
](
    size,
    vector,
    > Vector::map_base[A, B],
    > Vector::map_cont[A, B](call)
)
