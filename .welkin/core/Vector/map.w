map:
*    ~as A       |->
*    ~as B       |->
Size ~as length  |->
Vector[A, length] ->
(A -> B)          ->
Vector[B, length]

A ||>
B ||>
length ||>
vector |>
call |>
~match vector ~with size {
    nil = Vector::nil[B]
    cons[size](
        head, tail
    )   = Vector::cons[B, size](call(head), tail)
    : _ |> Vector[B, size]
}