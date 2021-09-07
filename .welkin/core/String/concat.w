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
as < as
bs < bs
vector < ~match as ~with size {
    new[o](av) = ~match bs ~with size {
        new[p](bv) = Vector::concat[Char](o, p, > av, > bv)
        : _ |> 'Vector[Char, Size::add(o, size)]
    }
    : _ |> 'Vector[Char, Size::add(size, m)]
}
> String::new[Size::add(n, m)](vector)