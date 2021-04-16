~data Vector (size: Nat) {
    nil(proof: Equal[.Nat, size, Nat::zero]),
    cons(length: Nat, head: Bool, tail: Vector[.length],
        proof: Equal[.Nat, size, Nat::succ[length]]
    )
}