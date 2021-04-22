induct:
Size        ~as n     ->
(Size -> *) ~as prop |->
'(
    Size ~as n ->
    prop(n)    ->
    prop(Size::succ(n))
)                     ->
'prop(Size::zero)     ->
'prop(n)

n |>
prop ||>
f |>
base |>
f < f
base < base
f < n[
    n |> prop(n)
](>f)
> f(base)