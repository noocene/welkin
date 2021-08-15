unfold:
*    ~as B     |->
*    ~as A     |->
Size ~as length ->
'A              ->
'(
    Size ~as n |->
    A           ->
    Pair[B, A]
)               ->
'Pair[Vector[B, length], A]

B ||> A ||>
length |>
initial |>
call |>
initial < initial
call < call
Size::induct[
    n |> Pair[Vector[B, n], A]
](
    length,
    > Pair::new[Vector[B, Size::zero], A](Vector::nil[A], initial),
    > n ||> pair |>
    (~match pair {
        new(vector, state) = ~match call[n](state) {
            new(element, new_state) =
                Pair::new[Vector[B, Size::succ(n)], A](
                    Vector::cons[B, n](element, vector),
                    new_state
                )
            : _ |> Pair[Vector[B, Size::succ(n)], A]
        }
        : _ |> Pair[Vector[B, Size::succ(n)], A]
    })
)