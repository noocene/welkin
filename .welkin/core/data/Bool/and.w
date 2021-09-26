and:
Bool ->
Bool ->
Bool

a |> b |>
~match a {
    true = ~match b {
        true  = Bool::true
        false = Bool::false
        : _ |> Bool
    }
    false  = Bool::false
    : _ |> Bool
}