increment:
Size ~as size |->
Word[size]     ->
Word[size]

size ||>
word |>
~match word ~with size {
    empty             = Word::empty
    low[size](after)  = Word::high[size](after)
    high[size](after) = Word::low[size](Word::increment[size](after))
    : _ |> Word[size]
}