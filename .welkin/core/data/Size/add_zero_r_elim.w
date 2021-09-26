add_zero_r_elim:
Size ~as n ->
'Equal[
    Size,
    Size::add(n, Size::zero),
    n
]

n |>
Size::induct[
    n |> Equal[Size, Size::add(n, Size::zero), n]
](
    n,
    > Equal::refl[Size, Size::zero],
    >
        n ||> h |>
        Equal::map[Size, Size, Size::add(n, Size::zero), n, Size::succ](h)
)