trivial_size:
'Size

> (~literal Size 7)

// shouldn't typecheck
transform:
* ~as A |->
* ~as B |->
A ->
B

_ ||> _ ||>
A |>
A

// this is fine
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

induct:
(Size -> *) ~as prop |->
Size        ~as n     ->
'prop(Size::zero)     ->
'(
// note this argument is no longer erased
    Size ~as n  ->
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
// but it's still accepted here. are we dropping erasure in the equality test?
](>f)
> f(initial)