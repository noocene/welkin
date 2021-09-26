print:
Sized[String] ->
WhelkIO[Unit, ~literal Size 1]

data |>
WhelkIO::call[Unit, Size::zero](
    WhelkRequest::print(data),
    resp |> WhelkIO::pure[Unit](resp)
)

prompt:
WhelkIO[Sized[String], ~literal Size 1]

WhelkIO::call[Sized[String], Size::zero](
    WhelkRequest::prompt(Unit::new),
    resp |> WhelkIO::pure[Sized[String]](resp)
)

once:
Size ~as steps      |->
Size ~as complexity |->
BoxPoly[
    WhelkIO[Unit, steps],
    complexity
]                    ->
WhelkIO[Unit, ~literal Size 1]

steps ||>
complexity ||>
operation |>
WhelkIO::call[Unit, Size::zero](
    WhelkRequest::loop[
        Unit,
        steps,
        complexity
    ](Unit::new, _ |> Bool::false, _ |> operation),
    _ |> WhelkIO::pure[Unit](Unit::new)
)

forever:
Size ~as steps      |->
Size ~as complexity |->
* ~as state         |->
(
    BoxPoly[
        state,
        complexity
    ] ->
    BoxPoly[
        WhelkIO[state, steps],
        complexity
    ]
)                    ->
state                ->
WhelkIO[Unit, ~literal Size 1]

steps ||>
complexity ||>
state ||>
operation |>
initial |>
WhelkIO::call[Unit, Size::zero](
    WhelkRequest::loop[
        state,
        steps,
        complexity
    ](initial, _ |> Bool::true, state |> operation(state)),
    _ |> WhelkIO::pure[Unit](Unit::new)
)

repeat_forever:
Size ~as steps      |->
Size ~as complexity |->
BoxPoly[
    WhelkIO[Unit, steps],
    complexity
]                    ->
WhelkIO[Unit, ~literal Size 1]

steps ||>
complexity ||>
operation |>
forever[
    steps,
    complexity,
    Unit
](_ |> operation, Unit::new)