// head:
// *   ~as A    |->
// Nat ~as size |->
// Vector[
//     A, Nat::succ(size)
// ]             ->
// A

// A ||>
// _ ||>
// vector |>
// ~match vector ~with size {
//     nil = Unit::new
//     cons[_](
//         head, _
//     )   = head
//     : _ |> ~match size {
//         zero    = Unit
//         succ(_) = A
//         : _ |> *
//     }
// }