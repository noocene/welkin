pure:
* ~as R    |->
(
    R ->
    *
) ~as resp |->
* ~as A    |->
A           ->
IO[R, resp, A, Size::zero]

R ||> resp ||>
A ||>
a |>
IO::end[R, resp, A](a)