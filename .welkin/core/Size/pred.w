pred:
Size ->
Size

n |>
P ||>
S |>
S < S
F < n[
    n |> Bool ~as b ->
    b[
        b |> *
    ](
        P(Size::pred(n)),
        P(Size::pred(Size::succ(n)))
    )
](>
    n ||> h |> b |>
    b[
        b |> P(Size::pred(Size::succ(n))) ~as h ->
        b[
            b |> *
        ](
            P(Size::pred(Size::succ(n))),
            P(Size::succ(Size::pred(Size::succ(n))))
        )
    ](
        x |> x,
        S[Size::pred(Size::succ(n))],
        h(Bool::false)
    )
)
> Z |> F(
	b |> b[
		b |> P(Size::zero) ->
        b[
		    b |> *
        ](
            P(Size::pred(Size::zero)),
            P(Size::pred(Size::succ(Size::zero)))
        )
    ](
        z |> z,
        z |> z,
        Z
    ),
    Bool::true
)