main:
Whelk

Whelk::new[~literal Size 1](
    data |>
    BoxPoly::new[
        Sized[String],
        ~literal Size 1
    ](> Sized::new[String](
        ~literal Size 5,
        ~literal String "hello"
    ))
)