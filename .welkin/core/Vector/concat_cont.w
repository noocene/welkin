concat_cont:
* ~as A    |->
Size ~as m |->
Size ~as n |->
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
n ||>
_ ||>
vector |>
cont |>
(~match vector ~with size {
    nil = _ |> _ |> _ |> Vector::nil[A]
    cons[size](
        head,
        tail
    )   = 
        ea |> eb |> c |>
        (~match Unit::new {
            new = ea |> eb |> c |>
                Vector::cons[A, Size::add(size, m)](
                    head,
                    Equal::rewrite[
                        Size,
                        n,
                        size,
                        n |> Vector[A, Size::add(n, m)]
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
                        A,
                        Size::add(n, m)
                    ]
                ) ->
                Vector[A, Size::add(Size::succ(size), m)]
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
                A,
                Size::add(
                    n,
                    m
                )
            ]
        ) ->
        is_zero < Size::is_zero(size)
        (~match is_zero {
            true  = Vector[A, Size::zero]
            false = Vector[A, Size::add(size, m)]
            : _ |> *
        })
})(Equal::refl[Size, Size::succ(n)], Equal::refl[Size, Size::succ(n)], cont[Unit::new])