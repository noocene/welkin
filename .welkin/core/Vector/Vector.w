~data Vector A ~with {
    size: Nat
} {
    nil[] ~with { Nat::zero },
    cons[length: Nat](
        head: A, 
        tail: Vector[A, length]
    )   ~with { Nat::succ(length) }
}