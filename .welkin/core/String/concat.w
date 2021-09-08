concat:
Size ~as n ->
Size ~as m ->
'String[n] ->
'String[m] ->
'String[Size::add(n, m)]

n |>
m |>
as |>
bs |>
vector < Vector::concat[Char](
    n,
    m,
    as < as
    > ~match as ~with size {
        new[_](vector) = vector
        : _ |> Vector[Char, size]
    },
    bs < bs
    > ~match bs ~with size {
        new[_](vector) = vector
        : _ |> Vector[Char, size]
    }
)
> String::new[Size::add(n, m)](vector)