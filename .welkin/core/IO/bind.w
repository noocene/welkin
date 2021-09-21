bind:
* ~as R      |->
(
    R ->
    *
) ~as resp   |->
Size ~as n    ->
Size ~as m    ->
* ~as A      |->
* ~as B      |->
'IO[
    R, resp, A, n
] ->
'(
    A ->
    IO[R, resp, B, m]
) ->
'IO[R, resp, B, Size::add(n, m)]

R ||> resp ||>
n |> m |>
A ||> B ||>
io |>
call |>
call < call
elim < Size::add_zero_l_elim(m)
Size::recurse[
    Unit,
    n |> _ |> Pair::new[*, *](
        IO[R, resp, A, n],
        IO[R, resp, B, Size::add(n, m)]
    ),
    Unit::new
](
    n,
    io,
    > IO::bind_base[R, resp, A, B, m](elim, call),
    > IO::bind_cont[R, resp, A, B, m]
)