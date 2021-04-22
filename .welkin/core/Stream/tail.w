tail:
* ~as A  |->
Stream[A] ->
Stream[A]

A ||>
stream |>
~match stream {
    new(_, tail) = tail(Unit::new)
    : _ |> Stream[A]
}