// map:
// * ~as A         |->
// * ~as B         |->
// Size ~as size    ->
// '(A -> B)        ->
// 'Vector[A, size] ->
// 'Vector[B, size]

// A ||>
// B ||>
// size |>
// call |>
// vector |>
// call < call
// Size::recurse[
//     Size,
//     n |> Pair::new[*, *](
//         Vector[A, n],
//         Vector[B, n]
//     ),
//     size
// ](
//     size,
//     vector,
//     > _ ||> a |> a,
//     >
//         size ||>
// 	    vector |>
// 	    cont |>
// 	    Vector::map_cont[
// 		    A,
//             B,
//             size
//         ](
//             call,
//             vector,
//             cont
//         )
// )
