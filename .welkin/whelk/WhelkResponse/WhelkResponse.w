WhelkResponse:
WhelkRequest ->
*

request |>
~match request {
    print(_)  = Unit
    prompt(_) = Sized[String]
    : _ |> *
}