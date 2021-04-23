pure:
* ~as S |->
* ~as A |->
A        ->
State[S, A]

S ||> A ||>
a |>
State::new[S, A](s |> Pair::new[A, S](a, s))