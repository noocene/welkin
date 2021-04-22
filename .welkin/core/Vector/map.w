map:
*    ~as A       |->
*    ~as B       |->
Size ~as length  |->
(A -> B)          ->
Vector[A, length] ->
Vector[B, length]

A ||>
B ||>
length ||>
call |>
vector |>
~match vector ~with size {
    nil = Vector::nil[B]
    cons[size](
        head, tail
    )   = Vector::cons[B, size](call(head), tail)
    : _ |> Vector[B, size]
}