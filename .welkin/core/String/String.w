~data String ~with { _: Size } {
    new[length: Size](
        value: Vector[Char, length]
    ) ~with { length }
}