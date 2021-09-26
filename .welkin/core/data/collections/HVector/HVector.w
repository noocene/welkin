~data HVector ~with {
    size: Size,
    _: Vector[*, size]
} {
    nil ~with { Size::zero, Vector::nil[*] },
    cons[length: Size, types: Vector[*, length], type: *](
        head: type, 
        tail: HVector[length, types]
    )   ~with { Size::succ(length), Vector::cons[*, length](type, types) }
}