take:
*    ~as A      |->
Size ~as length  ->
'Stream[A]       ->
'Pair[Vector[A, length], Stream[A]]

A ||>
length |>
stream |>
Vector::unfold[A, Stream[A]](
    length,
    stream,
    > _ |> stream |> Stream::open[A](stream)
)