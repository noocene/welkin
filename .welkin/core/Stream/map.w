// TODO: not stratified

// map:
// * ~as A  |->
// * ~as B  |->
// (A -> B)  ->
// Stream[A] ->
// Stream[B]

// A ||>
// B ||>
// call |>
// stream |>
// ~match stream {
//     new(head, tail) = Stream::new[B](call(head), _ |> Stream::map[A, B](call, tail(Unit::new)))
//     : _ |> Stream[B]
// }