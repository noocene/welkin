monad:
* ~as S |->
Monad[A |> State[S, A]]

S ||>
Monad::new[A |> State[S, A]](
    State::bind[S],
    State::pure[S]
)