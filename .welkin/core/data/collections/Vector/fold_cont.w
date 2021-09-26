fold_cont:
* ~as A    |->
* ~as B    |->
(
    B ->
    A ->
    B
)           ->
Size ~as n |->
Unit       |->
Pair[B, Vector[
    A, Size::succ(n)
]]          ->
(
    Unit |->
    Pair[B, Vector[
        A, n
    ]]    ->
    B
)           ->
B

A ||> B ||>
call |>
n ||> _ ||>
pair |>
cont |>
~match pair {
    new(accumulator, vector) = (~match vector ~with size {
        nil = _ |> Unit::new
        cons[size](
            head,
            tail
        )   = e |> (~match Unit::new {
                new = e |> cont[Unit::new](
                    Pair::new[
                        B,
                        Vector[A, n]
                    ](
                        call(accumulator, head),
                        Equal::rewrite[
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
                        ](e), tail)
                    )
                )
                : _ |>
                    Equal[
                        Size,
                        Size::succ(size),
                        Size::succ(n)
                    ] ->
                    B
            })(e)
        : _ |>
            Equal[
                Size,
                size,
                Size::succ(n)
            ] ->
            is_zero < Size::is_zero(size)
            (~match is_zero {
                true  = Unit
                false = B
                : _ |> *
            })
    })(Equal::refl[Size, Size::succ(n)])
    : _ |> B
}