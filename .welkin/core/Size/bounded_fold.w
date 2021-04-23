bounded_fold:
* ~as A  |->
Size      ->
''A       ->
'(
    'A -> Pair['A, Bool]
)         ->
''A

A ||>
n |>
initial |>
call |>
initial < initial
call < call
f < n[
    x |> Pair['A, Bool]
](>
    n ||> pair |>
    ~match pair {
        new(data, continue) =
            data < data
            (~match continue {
                true  = call(> data)
                false = Pair::new['A, Bool](> data, Bool::false)
                : _ |> Pair['A, Bool]
            })
        : _ |> Pair['A, Bool]
    }
)
> ~match f(Pair::new['A, Bool](initial, Bool::true)) {
    new(data, _) = data
    : _ |> 'A
}