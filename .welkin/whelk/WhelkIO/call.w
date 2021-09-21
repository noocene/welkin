call:
* ~as A           |->
Size ~as n        |->
WhelkRequest ~as r ->
(
    WhelkResponse(r) ->
    WhelkIO[A, n]
)                  ->
WhelkIO[A, Size::succ(n)]

A ||> n ||>
r |> call |>
IO::call[WhelkRequest, WhelkResponse, A, n](r, call)