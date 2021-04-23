semigroup:
* ~as A  |->
Monoid[A] ->
Semigroup[A]

A ||>
monoid |>
~match monoid {
    new(semigroup, _) = semigroup
    : _ |> Semigroup[A]
}