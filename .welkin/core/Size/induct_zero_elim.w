induct_zero_elim:
(Size -> *) ~as prop         |->
'prop(Size::zero) ~as initial ->
'(
    Size ~as n |->
    prop(n)     ->
    prop(Size::succ(n))
) ~as call                    ->
Equal[
    'prop(Size::zero),
    Size::induct[
        prop
    ](
        Size::zero,
        initial,
        call
    ),
    initial
]

prop ||>
initial |>
_ |>
Equal::refl['prop(Size::zero), initial]