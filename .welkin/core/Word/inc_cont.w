inc_cont:
Size ~as n |->
Unit       |->
Word[
    Size::succ(n)
]           ->
(
    Unit   |->
    Word[n] ->
    Word[n]
)           ->
Word[
    Size::succ(n)
]

n ||>
_ ||>
word |>
cont |>
(~match word ~with size {
    empty    = _ |> _ |> Word::empty
    low[
        size
    ](after) = _ |> _ |> Word::high[size](after)
    high[
        size
    ](after) = ea |> eb |>
        (~match Unit::new {
            new = ea |> eb |>
                Word::low[size](
                    Equal::rewrite[
                        Size,
                        n,
                        size,
                        n |> Word[n]
                    ](Equal::flip[
                        Size,
                        size,
                        n
                    ](Equal::map[
                        Size,
                        Size,
                        Size::succ(size),
                        Size::succ(n),
                        n |> 
                            pred < Size::extract_pred[Size](
                                n,
                                > n ||> n
                            )
                            pred
                    ](eb)), cont[Unit::new](
                    Equal::rewrite[
                        Size,
                        size,
                        n,
                        n |> Word[n]
                    ](Equal::map[
                        Size,
                        Size,
                        Size::succ(size),
                        Size::succ(n),
                        n |> 
                        pred < Size::extract_pred[Size](
                            n,
                            > n ||> n
                        )
                        pred
                    ](ea), after)
                )))
                : _ |>
                    Equal[
                        Size,
                        Size::succ(size),
                        Size::succ(n)
                    ] ->
                    Equal[
                        Size,
                        Size::succ(size),
                        Size::succ(n)
                    ] ->
                    Word[Size::succ(size)]
        })(ea, eb)
    : _ |>
        Equal[
            Size,
            size,
            Size::succ(n)
        ] ->
        Equal[
            Size,
            size,
            Size::succ(n)
        ] ->
        Word[size]
})(Equal::refl[Size, Size::succ(n)], Equal::refl[Size, Size::succ(n)])