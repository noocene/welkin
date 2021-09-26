ones:
Size ~as length ->
'Word[length]

length |>
Size::induct[
    n |> Word[n]
](
    length,
    > Word::empty,
    > n ||> word |> Word::high[n](word)
)