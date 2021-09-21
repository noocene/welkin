bind:
* ~as A       |->
* ~as B       |->
Size ~as n     ->
Size ~as m     ->
'WhelkIO[A, n] ->
'(
    A ->
    WhelkIO[B, m]
)              ->
'WhelkIO[B, Size::add(n, m)]

A ||> B ||>
n |> m |>
io |> call |>
(IO::bind[WhelkRequest, WhelkResponse](n, m))[A, B](io, call)

