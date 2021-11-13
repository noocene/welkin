y:
* ~as A |->
(
    A |-> A
) -> A

A ||>
f |>
(~match Unit::new {
    new = x |> f[(Rec::out[A](x))[x]]
    : _ |> Rec[A] -> A
})(
    Rec::in[A](
        x ||>
        f[(Rec::out[A](x))[x]]
    )
)