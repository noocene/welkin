or:
Bool ->
Bool ->
Bool

a |> b |>
~match a {
    true  = Bool::true
    false = ~match b {
        true  = Bool::true
        false = Bool::false
        : _ |> Bool
    }
    : _ |> Bool
}