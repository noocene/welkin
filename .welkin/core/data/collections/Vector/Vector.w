~data Vector A ~with {
    _: Size
} {
    nil ~with { Size::zero },
    cons[size: Size](
        head: A, 
        tail: Vector[A, size]
    )   ~with { Size::succ(size) }
}