bind:
* ~as S    |->
* ~as A    |->
* ~as B    |->
State[S, A] ->
(
    A -> State[S, B]
)           ->
State[S, B]

S ||> A ||> B ||>
state |>
call |>
~match state {
    new(run) = State::new[S, B](
        s |>
        ~match run(s) {
            new(value, state) = ~match call(value) {
                new(run) = run(state)
                : _ |> Pair[B, S]
            }
            : _ |> Pair[B, S]
        }
    )
    : _ |> State[S, B]
}