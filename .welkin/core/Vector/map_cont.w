map_cont:
* ~as A        |->
* ~as B        |->
Size ~as size  |->
(A -> B)        ->
Vector[A, size] ->
(
    Size ~as size  |->
    Vector[A, size] ->
    Vector[B, size]
)               ->
Vector[B, size]

A ||>
B ||>
size ||>
call |>
vector |>
cont |>
~match vector ~with size {
    nil = Vector::nil[B]
    cons[size](
        head,
        tail
    )   = Vector::cons[B, size](
        call(head),
        cont[size](tail)
    )
    : _ |> Vector[B, size]
}