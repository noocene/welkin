main:
Whelk

data < WhelkIO::bind[
    Sized[String],
    Unit
](
    ~literal Size 1,
    ~literal Size 1,
    > prompt,
    >
        data |>
        print(data)
)

Whelk::new[
    ~literal Size 1,
    ~literal Size 0
](
    BoxPoly::new[
        WhelkIO[Unit, ~literal Size 1],
        ~literal Size 0
    ](repeat_forever[
        ~literal Size 2,
        ~literal Size 1
    ](BoxPoly::new[
        WhelkIO[Unit, ~literal Size 2],
        ~literal Size 1
    ](> data)))
)