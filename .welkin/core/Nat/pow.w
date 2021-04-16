pow:
Nat ->
Nat ->
Nat

a |> b |>
~match b {
    zero       = Nat::zero
    succ(pred) = Nat::mul(a, Nat::pow(a, pred))
    : Nat
}