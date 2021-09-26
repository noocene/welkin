induct:
(Size -> *) ~as prop |->
Size        ~as n     ->
'prop(Size::zero)     ->
'(
    Size ~as n |->
    prop(n)     ->
    prop(Size::succ(n))
)                     ->
'prop(n)

prop ||>
n |>
initial |>
call |>
initial < initial
call < call
f < n[
    n |> prop(n)
](>
    n ||> h |>
    call[n](h)
)
> f(initial)