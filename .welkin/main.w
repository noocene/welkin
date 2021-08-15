// \x \x (^1 (^1 (^1 (^1 (^1 (^1 (^1 ^0)))))))
main:
'Size

// Size::succ(Size::zero)

Size::induct[
    _ |> Size
](
    (~literal Size 3),
    > (~literal Size 4),
    > _ ||> size |> Size::succ(size)
)

// Size::fold[Size](
//     ~literal Size 4,
//     > (~literal Size 3),
//     > n |> Size::succ(n)
// )

other:
'Word[~literal Size 3]

> Word::high[~literal Size 2](Word::high[~literal Size 1](Word::high[Size::zero](Word::empty)))

// same:
// Equal['Word[~literal Size 3], main, other]

// Equal::refl['Word[~literal Size 3], main]

transform:
* ~as A |->
* ~as B |->
A ->
B

_ ||> _ ||>
A |>
A

number_pair:
'Size ->
'Pair[Size, Size]

size |>
size < size
> Pair::new[Size, Size](size, size)