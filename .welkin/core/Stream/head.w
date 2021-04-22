head:
* ~as A  |->
Stream[A] ->
A

A ||>
stream |>
~match stream {
    new(head, _) = head
    : _ |> A
}