mul : Nat -> Nat -> Nat
    a => b =>
    ~match b {
        zero       = Nat::zero
        succ(pred) = Nat::add[a, Nat::mul[a, pred]]
        : Nat
    }