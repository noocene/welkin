// inc_cont:
// Size ~as n |->
// Unit       |->
// Word[
//     Size::succ(n)
// ]           ->
// (
//     Unit   |->
//     Word[n] ->
//     Word[n]
// )           ->
// Word[
//     Size::succ(n)
// ]

// size ||>
// _ ||>
// word |>
// cont |>
// ~match word ~with n {
//     empty    = Word::empty
//     low[
//         _
//     ](after) = Word::high[size](after)
//     high[
//         _
//     ](after) = Word::low[size](after)
//     : _ |> Word[n]
// }