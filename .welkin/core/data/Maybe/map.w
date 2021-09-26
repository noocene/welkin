map:
* ~as A |->
* ~as B |->
(A -> B) ->
Maybe[A] ->
Maybe[B]

A ||>
B ||>
call |>
maybe |>
~match maybe {
    nothing     = Maybe::nothing[B]
    just(value) = Maybe::just[B](call(value))
    : _ |> Maybe[B]
}