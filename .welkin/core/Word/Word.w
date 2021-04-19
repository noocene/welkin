~data Word ~with {
    _: Size
} {
    empty ~with { Size::zero },
    low[size: Size](after: Word[size])  ~with { Size::succ(size) },
    high[size: Size](after: Word[size]) ~with { Size::succ(size) }
}