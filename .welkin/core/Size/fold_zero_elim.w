fold_zero_elim:
* ~as A       |->
'A ~as initial ->
'(
    A -> A
) ~as call     ->
Equal[
    'A,
    Size::fold[A](
        Size::zero,
        initial,
        call
    ),
    initial
]

A ||>
initial |>
_ |>
Equal::refl['A, initial]