from_size:
Size ~as size  ->
Size           ->
'Word[size]

size |>
n |>
Size::fold[Word[size]](
    n,
    Word::zeros(size),
    > word |> Word::increment[size](word)
)