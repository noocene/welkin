fold:
* ~as A  |->
Size      ->
'A        ->
'(A -> A) ->
'A

A ||>
n |>
initial |>
call |>
call < call
Size::induct[
    n |> A
](
    n,
    initial,
    > n ||> call
)