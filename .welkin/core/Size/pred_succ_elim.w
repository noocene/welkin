pred_succ_elim:
Size ~as n ->
'Equal[Size, Size::pred(Size::succ(n)), n]

n |>
Size::induct[
    n |> Equal[Size, Size::pred(Size::succ(n)), n]
](
    n,
    > Equal::refl[Size, Size::zero],
    >
        n ||> h |>
        Equal::map[Size, Size, Size::pred(Size::succ(n)), n, Size::succ](h)
)