// TODO: depends on induct_indexed, which is not stratified
// recurse_indexed:
// * ~as A              |->
// (
//     Size ->
//     A    ->
//     Pair[*, *]
// ) ~as prop           |->
// A ~as top            |->
// Size ~as n            ->
// 'Pair::left[*, *](
//     prop(n, top)
// )                     ->
// '(
//     A ~as state |->
//     Pair::left[*, *](
//         prop(Size::zero, state)
//     )            ->
//     Pair::right[*, *](
//         prop(Size::zero, state)
//     )
// )                     ->
// '(
//     Size ~as n   ->
//     A ~as state |->
//     Pair::left[*, *](
//         prop(Size::succ(n), state)
//     )            -> 
//     (
//         A ~as state |->
//         Pair::left[*, *](
//             prop(n, state)
//         )            -> 
//         Pair::right[*, *](
//             prop(n, state)
//         )
//     )            -> 
//     Pair::right[*, *](
//         prop(Size::succ(n), state)
//     )
// )                     ->
// 'Pair::right[*, *](
//     prop(n, top)
// )

// A ||>
// prop ||>
// top ||>
// size |>
// initial |>
// base |>
// call |>
// call < call
// initial < initial
// call < Size::induct_indexed[
//     n |>
//         A ~as state |->
//         Pair::left[*, *](
//             prop(n, state)
//         )            -> 
//         Pair::right[*, *](
//             prop(n, state)
//         )
// ](
//     size,
//     base,
//     > n |> cont |> state ||> data |> (call(n))[state](data, cont)
// )
// > call[top](initial)
