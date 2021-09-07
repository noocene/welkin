concat_cont:
* ~as A    |->
Size ~as m |->
Size ~as n  ->
Unit       |->
Vector[
    A, Size::succ(n)
]           ->
(
    Unit        |->
    Vector[A, n] ->
    Vector[A, Size::add(n, m)]
)           ->
Vector[
    A, Size::add(Size::succ(n), m)
]

A ||>
m ||>
n  |>
_ ||>
vector |>
cont |>
elim < Size::pred_succ_elim(n)
(~match vector ~with size {
    nil = _ |> Vector::nil[A]
    cons[size](
        head,
        tail
    )   = 
        elim < Size::pred_succ_elim(size)
        c |> Vector::cons[A, Size::add(size, m)](
            head,
            Equal::rewrite[
                Size,
                Size::pred(Size::succ(size)),
                size,
                size |> Vector[A, Size::add(size, m)]
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
    : _ |>
        (
            Vector[
                A,
                Size::pred(size)
            ]     ->
            Vector[
                A,
                Size::add(Size::pred(size), m)
            ]
        ) ->
        is_zero < Size::is_zero(size)
        (~match is_zero {
            true  = Vector[A, Size::zero]
            false = Vector[A, Size::add(size, m)]
            : _ |> *
        })
})(
    vector |>
    Equal::rewrite[
        Size,
        n,
        Size::pred(Size::succ(n)),
        size |> Vector[A, Size::add(size, m)]
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