cons:
Size ~as length |->
String[length]   ->
Char             ->
String[Size::succ(length)]

_ ||>
string |>
char |>
~match string ~with size {
    new[length](data) = String::new[
        Size::succ(length)
    ](Vector::cons[Char, length](char, data))
    : _ |> String[Size::succ(size)]
}