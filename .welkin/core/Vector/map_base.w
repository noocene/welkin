map_base:
* ~as A |->
* ~as B |->
Unit    |->
Vector[
    A, Size::zero
]        ->
Vector[
    B, Size::zero
]

A ||>
B ||>
_ ||>
vector |>
~match vector ~with size {
    nil = Vector::nil[B]
    cons[_](
        _,
        _
    )   = Unit::new
    : _ |>
        is_zero < Size::is_zero(size)
        (~match is_zero {
            true = Vector[B, Size::zero]
            false = Unit
            : _ |> *
        })
}