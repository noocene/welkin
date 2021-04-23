monad:
* ~as S |->
Monad[State[S]]

S ||>
Monad::new[State[S]](
    State::bind[S],
    State::pure[S]
)