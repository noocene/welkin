take:
*    ~as A      |->
Size ~as length  ->
'Stream[A]        ->
'Pair[Vector[A, length], Stream[A]]

A ||>
length |>
stream |>
stream < stream
Size::induct[
    n |> Pair[Vector[A, n], Stream[A]]
](
    length,
    > n |> pair |>
    ~match pair {
        new(vector, stream) = ~match Stream::open[A](stream) {
            new(head, tail) = Pair::new[
                Vector[A, Size::succ(n)], Stream[A]
            ](
                Vector::cons[A, n](
                    head,
                    vector
                ),
                tail
            )
            : _ |> Pair[Vector[A, Size::succ(n)], Stream[A]]
        }
        : _ |> Pair[Vector[A, Size::succ(n)], Stream[A]]
    },
    > Pair::new[Vector[A, Size::zero], Stream[A]](Vector::nil[A], stream)
)