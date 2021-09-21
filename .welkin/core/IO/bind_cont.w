bind_cont:
* ~as R    |->
(
    R ->
    *
) ~as resp |->
* ~as A    |->
* ~as B    |->
Size ~as m |->
Size ~as n |->
Unit       |->
IO[
    R, resp, A, Size::succ(n)
]           ->
(
    Unit             |->
    IO[R, resp, A, n] ->
    IO[R, resp, B, Size::add(n, m)]
)           ->
IO[
    R, resp, B, Size::add(Size::succ(n), m)
]

R ||> resp ||>
A ||> B ||>
m ||>
n ||>
_ ||>
io |>
cont |>
(~match io ~with size {
    end(_) = _ |> _ |> _ |> Unit::new
    call[size](
        request,
        then
    )      = 
        ea |> eb |> c |>
        (~match Unit::new {
            new = ea |> eb |> c |>
                IO::call[R, resp, B, Size::add(size, m)](
                    request,
                    rr |>
                        Equal::rewrite[
                            Size,
                            n,
                            size,
                            n |> IO[R, resp, B, Size::add(n, m)]
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
                            n |> IO[R, resp, A, n]
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
                        ](ea), then(rr))))
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
                    IO[
                        R,
                        resp,
                        A,
                        n
                    ]     ->
                    IO[
                        R,
                        resp,
                        B,
                        Size::add(n, m)
                    ]
                ) ->
                IO[R, resp, B, Size::add(Size::succ(size), m)]
        })(ea, eb, c)
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
        (
            IO[
                R,
                resp,
                A,
                n
            ]     ->
            IO[
                R,
                resp,
                B,
                Size::add(
                    n,
                    m
                )
            ]
        ) ->
        is_zero < Size::is_zero(size)
        (~match is_zero {
            true  = Unit
            false = IO[R, resp, B, Size::add(size, m)]
            : _ |> *
        })
})(Equal::refl[Size, Size::succ(n)], Equal::refl[Size, Size::succ(n)], cont[Unit::new])