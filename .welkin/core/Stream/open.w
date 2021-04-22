open:
* ~as A  |->
Stream[A] ->
Pair[A, Stream[A]]

A ||>
stream |>
~match stream {
    new(head, tail) = Pair::new[A, Stream[A]](head, tail(Unit::new))
    : _ |> Pair[A, Stream[A]]
}