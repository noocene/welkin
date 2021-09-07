induct_indexed:
(Size -> *) ~as prop |->
Size        ~as n     ->
'prop(Size::zero)     ->
'(
    Size ~as n  ->
    prop(n)     ->
    prop(Size::succ(n))
)                     ->
'prop(n)

prop ||>
n |>
initial |>
call |>
initial < initial
call < call

pair < Size::induct[
    n |> Pair[prop(n), Sigma[Size, index |> Equal[Size, n, index]]]
](
    n,
    > Pair::new[
        prop(Size::zero),
        Sigma[
            Size,
            index |> Equal[Size, Size::zero, index]
        ]
    ](
        initial,
        Sigma::new[
            Size,
            index |> Equal[Size, Size::zero, index]
        ](
            Size::zero,
            Equal::refl[Size, Size::zero]
        )
    ),
    > n ||> pair |>
    ~match pair {
        new(data, index) = ~match index {
            new(index, proof) = Pair::new[prop(Size::succ(n)), Sigma[Size, index |> Equal[Size, Size::succ(n), index]]](
                Equal::rewrite[
                    Size,
                    index,
                    n,
                    n |> prop(Size::succ(n))
                ](Equal::flip[
                    Size,
                    n,
                    index
                ](proof), call(index, Equal::rewrite[
                    Size,
                    n,
                    index,
                    n |> prop(n)
                ](proof, data))),
                Sigma::new[
                    Size,
                    index |> Equal[Size, Size::succ(n), index]
                ](
                    Size::succ(index),
                    Equal::map[
                        Size,
                        Size,
                        n,
                        index,
                        n |> Size::succ(n)
                    ](proof)
                )
            )
            : _ |> Pair[prop(Size::succ(n)), Sigma[Size, index |> Equal[Size, Size::succ(n), index]]]
        }
        : _ |> Pair[prop(Size::succ(n)), Sigma[Size, index |> Equal[Size, Size::succ(n), index]]]
    }
)
> Pair::left[prop(n), Sigma[Size, index |> Equal[Size, n, index]]](pair)