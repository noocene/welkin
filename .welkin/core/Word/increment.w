// increment:
// Size ~as size ->
// 'Word[size]   ->
// 'Word[size]

// size |>
// word |>
// Size::recurse[
//     Unit,
//     n |> _ |> Pair::new[*, *](Word[n], Word[n]),
//     Unit::new
// ](
//     size,
//     word,
//     > _ ||> a |> a,
//     > Word::inc_cont
// )