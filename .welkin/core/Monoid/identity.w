identity:
* ~as A  |->
Monoid[A] ->
A

A ||>
monoid |>
~match monoid {
    new(_, identity) = ~match identity {
        new(identity, _) = identity
        : _ |> A
    }
    : _ |> A
}