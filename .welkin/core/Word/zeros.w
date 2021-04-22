zeros:
Size ~as length ->
'Word[length]

length |>
Size::induct[
    n |> Word[n]
](
    length,
    > n |> word |> Word::low[n](word),
    > Word::empty
)