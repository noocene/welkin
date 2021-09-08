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
)           ->
Vector[
    B, Size::succ(n)
]


A ||> B ||>
conv |>
n ||> _ ||>
vector |>
cont |>
(~match vector ~with size {
    nil = _ |> _ |> _ |> Vector::nil[B]
    cons[size](
        head,
        tail
    )   = 
        ea |> eb |> c |>
        (~match Unit::new {
            new = ea |> eb |> c |>
                Vector::cons[B, size](
                    conv(head),
                    Equal::rewrite[
                        Size,
                        n,
                        size,
                        n |> Vector[B, n]
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
                    ](eb)), c(Equal::rewrite[
                        Size,
                        size,
                        n,
                        n |> Vector[A, n]
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
                    ](ea), tail)))
                    )
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
                (
                    Vector[
                        A,
                        n
                    ]     ->
                    Vector[
                        B,
                        n
                    ]
                ) ->
                Vector[B, Size::succ(size)]
        })(ea, eb, c)
    : vector |>
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
        (
            Vector[
                A,
                n

            ]     ->
            Vector[
                B,
                n
            ]
        ) ->
        is_zero < Size::is_zero(size)
        (~match is_zero {
            true  = Vector[B, Size::zero]
            false = Vector[B, size]
            : _ |> *
        })
})(Equal::refl[Size, Size::succ(n)], Equal::refl[Size, Size::succ(n)], cont[Unit::new])