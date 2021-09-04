size:
Bound ->
Size

bound |>
~match bound {
    new[_](size, _) = size
    : _ |> Size
}