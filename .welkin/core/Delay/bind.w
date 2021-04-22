// TODO: this isn't stratified

// bind:
// * ~as A |->
// * ~as B |->
// Delay[A] ->
// (
//     A -> Delay[B]
// )        ->
// Delay[B]

// A ||>
// B ||>
// delay |>
// call |>
// ~match delay {
//     now(value)   = call(value)
//     later(thunk) = Delay::later[B](
//         _ |> Delay::bind[A, B](thunk(Unit::new), call)
//     )
//     : _ |> Delay[B]
// }