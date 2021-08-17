main:
'Size

Size::induct[
    _ |> Size
](
    (~literal Size 3),
    > (~literal Size 4),
    > _ ||> size |> Size::succ(size)
)