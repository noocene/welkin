not:
Bool -> 
Bool

bool |>
~match bool {
    true  = Bool::false
    false = Bool::true
    : _ |> Bool
}