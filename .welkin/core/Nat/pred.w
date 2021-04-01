pred : Nat -> Nat
    nat =>
    ~match nat {
        zero       = Nat::zero
        succ(pred) = pred
        : Nat
    }