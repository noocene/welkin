~data Word (bits: Nat) {
    nil(proof: Equal[.Nat, bits, Nat::zero]),
    low(size: Nat, after: Word[.size],
        proof: Equal[.Nat, bits, Nat::succ[size]]
    ),
    high(size: Nat, after: Word[.size],
        proof: Equal[.Nat, bits, Nat::succ[size]]
    )
}