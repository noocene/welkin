~data IO R [resp: R -> *] A ~with {
    _: Size
} {
    end(value: A) ~with { Size::zero },
    call[size: Size](
        request: R,
        then:
            resp(request) ->
            IO[R, resp, A, size]
    )             ~with { Size::succ(size) }
}