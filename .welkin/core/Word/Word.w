~data Word ~with {
    with: Nat
} {
    empty ~with { Nat::zero },
    low(size: Nat, after: Word[size]) ~with { Nat::succ(size) },
    high(size: Nat, after: Word[size]) ~with { Nat::succ(size) }
}