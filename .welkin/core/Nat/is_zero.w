is_zero:
Nat -> 
Bool

nat |>
~match nat {
    zero    = Bool::true
    succ(_) = Bool::false
    : _ |> Bool
}