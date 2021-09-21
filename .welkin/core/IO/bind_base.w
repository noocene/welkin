bind_base:
* ~as R          |->
(
    R ->
    *
) ~as resp       |->
* ~as A          |->
* ~as B          |->
Size ~as m       |->
Equal[
    Size,
    Size::add(Size::zero, m),
    m
]                 ->
(
    A ->
    IO[R, resp, B, m]
)                 ->
Unit             |->
IO[
    R, resp, A, Size::zero
]                 ->
IO[
    R, resp, B, Size::add(Size::zero, m)
]

R ||> resp ||>
A ||> B ||>
m ||>
elim |>
call |>
_ ||>
io |>
~match io ~with size {
    end(value) = Equal::rewrite[
            Size,
            m,
            Size::add(Size::zero, m),
            n |> IO[R, resp, B, n]
        ](Equal::flip[
            Size,
            Size::add(Size::zero, m),
            m
        ](elim), call(value))
    call[_](
        _, _
    )          = Unit::new
    : _ |>
        is_zero < Size::is_zero(size)
        (~match is_zero {
            true  = IO[R, resp, B, Size::add(Size::zero, m)]
            false = Unit
            : _ |> *
        })
}