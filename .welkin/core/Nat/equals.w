equals : Nat -> Nat -> Bool
    a => b =>
    ~match a {
        zero             = ~match b {
            zero         = Bool::true
            succ(_)      = Bool::false
            : Bool
        }
        succ(a_pred)     = ~match b {
            zero         = Bool::false
            succ(b_pred) = Nat::equals[a_pred, b_pred]
            : Bool
        }
        : Bool
    }