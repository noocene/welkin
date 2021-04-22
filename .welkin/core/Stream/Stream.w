~data Stream A {
    new(
        head: A,
        tail: Thunk[Stream[A]]
    )
}