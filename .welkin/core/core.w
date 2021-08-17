void:
* ~as A |->
Void     ->
A

A ||>
void |>
~match void { : _ |> A }

compose:
* ~as A |->
* ~as B |->
* ~as C |->
(B -> C) ->
(A -> B) ->
(A -> C)

_ ||> _ ||> _ ||>
a |> b |>
x |> a(b(x))

identity:
* ~as A |->
A        ->
A

_ ||>
a |> a

unerase:
* ~as A |->
* ~as B |->
(
    A |-> B
)        ->
(
    A -> B
)

_ ||> _ ||>
call |>

arg |>
call[arg]