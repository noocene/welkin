add_zero_l_elim:
Size ~as n ->
'Equal[
    Size,
    Size::add(Size::zero, n),
    n
]

n |>
Size::induct[
    n |> Equal[Size, Size::add(Size::zero, n), n]
](
    n,
    > Equal::refl[Size, Size::zero],
    >
        n ||> h |>
        Equal::map[Size, Size, Size::add(Size::zero, n), n, Size::succ](h)
)