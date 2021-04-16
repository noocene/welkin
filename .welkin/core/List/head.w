head: 
* ~as A |-> 
List[A] ->
Maybe[A]

A ||>
list |>
~match list {
    nil = Maybe::nothing[A]
    cons(
        head, _
    )   = Maybe::just[A](head)
    : Maybe[A]
}