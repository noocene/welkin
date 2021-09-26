equal:
'Bool ->
'Bool ->
'Bool

a |> b |>
a < a
b < b
> ~match a {
    true  = b
    false = Bool::not(b)
    : _ |> Bool
}