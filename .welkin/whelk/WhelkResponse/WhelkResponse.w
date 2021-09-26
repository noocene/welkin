WhelkResponse:
WhelkRequest ->
*

request |>
~match request {
    print(_)  = Unit
    prompt(_) = Sized[String]
    loop[
        state, _, _
    ](_, _, _)   = state
    : _ |> *
}