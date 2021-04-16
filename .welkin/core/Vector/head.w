// TODO make this dependent (don't allow calling it on zero-len)
head:
*   ~as A    |->
Nat ~as size |->
Vector[
    A, size
]             ->
Maybe[A]

A ||>
size ||>
vector |>
~match vector ~with _ {
    nil = Maybe::nothing[A]
    cons[_](
        head, _
    )   = Maybe::just[A](head)
    : Maybe[A]
}