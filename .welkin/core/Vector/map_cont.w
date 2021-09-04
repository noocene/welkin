map_cont:
* ~as A    |->
* ~as B    |->
(A -> B)    ->
Size ~as n |->
Unit       |->
Vector[
	A, Size::succ(n)
]           ->
(
    Unit        |->
    Vector[A, n] ->
    Vector[B, n]
)               ->
Vector[
    B, Size::succ(n)
]


A ||> B ||>
conv |>
n ||> _ ||>
vector |>
cont |>
elim < Size::pred_succ_elim(n)
(~match vector ~with size {
    nil = _ |> Vector::nil[B]
    cons[size](
        head,
        tail
    )   = 
        elim < Size::pred_succ_elim(size)
        c |> Vector::cons[B, size](
            conv(head),
	        Equal::rewrite[
                Size,
                Size::pred(Size::succ(size)),
                size,
                size |> Vector[B, size]
            ](elim, c(Equal::rewrite[
                Size,
                size,
                Size::pred(Size::succ(size)),
                size |> Vector[A, size]
            ](Equal::flip[
                Size,
                Size::pred(Size::succ(size)),
                size
            ](elim), tail)))
    	)
    : tester |>
        (
            Vector[
                A,
                Size::pred(size)
            ]     ->
            Vector[
                B,
                Size::pred(size)
            ]
        ) ->
        Vector[B, size]
})(
    vector |>
    Equal::rewrite[
        Size,
        n,
        Size::pred(Size::succ(n)),
        size |> Vector[B, size]
    ](Equal::flip[
        Size,
        Size::pred(Size::succ(n)),
        n
    ](elim), cont[Unit::new](Equal::rewrite[
        Size,
        Size::pred(Size::succ(n)),
        n,
        size |> Vector[A, size]
    ](elim, vector)))
)