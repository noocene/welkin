// tail:
// * ~as A      |->
// Nat ~as size |->
// Vector[
//     A, Nat::succ(size)
// ]             ->
// Vector[A, size]

// A ||>
// _ ||>
// vector |>
// ~match vector ~with size {
//     nil = Unit::new
//     cons[_](
//         _, tail
//     )   = tail
//     : _ |> ~match size {
//         zero    = Unit
//         succ(_) = Vector[A, Nat::pred(size)]
//         : _ |> *
//     }
// }