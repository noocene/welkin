induct:
(Bound -> *) ~as prop |->
Bound        ~as n     ->
'prop(Bound::zero)     ->
'(
    Bound ~as n |->
    prop(n)      ->
    prop(Bound::succ(n))
)                      ->
// need to prove an eliminator for this
'prop(Bound::from_size(Bound::size(n)))

prop ||>
n |>
initial |>
call |>
call < call
Size::induct[
    n |> prop(Bound::from_size(n))
](
    Bound::size(n),
    initial,
    > n ||> h |>
    call[Bound::from_size(n)](h)
)