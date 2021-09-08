// TODO: figure this out (think the size needs to not be behind the box)
// concat_sized:
// 'Sized[String] ->
// 'Sized[String] ->
// 'Sized[String]

// a |>
// b |>
// a < a
// b < b
// n <> ~match a {
//     new(idx, _) = idx
//     : _ |> Size
// }
// m <> ~match b {
//     new(idx, _) = idx
//     : _ |> Size
// }
// a <> ~match a {
//     new(_, data) = data
//     : a |> String[~match a {
//         new(idx, _) = idx
//         : _ |> Size
//     }]
// }
// b <> ~match b {
//     new(_, data) = data
//     : b |> String[~match b {
//         new(idx, _) = idx
//         : _ |> Size
//     }]
// }
// string < String::concat(
//     n,
//     m,
//     > a,
//     > b
// )
// > Sized::new[String](Size::add(n, m), string)