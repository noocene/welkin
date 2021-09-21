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