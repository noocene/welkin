WhelkResponse:
WhelkRequest ->
*

request |>
~match request {
    print(_)   = Unit
    prompt(_)  = Sized[String]
    define(
        _, _, _
    )          = Unit
    loop[
        state, _, _
    ](_, _, _) = state
    : _ |> *
}