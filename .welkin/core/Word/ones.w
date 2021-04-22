ones:
Size ~as length ->
'Word[length]

length |>
Size::induct[
    n |> Word[n]
](
    length,
    > n |> word |> Word::high[n](word),
    > Word::empty
)