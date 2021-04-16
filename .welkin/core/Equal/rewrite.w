rewrite : * ~as A -> A ~as a -> A ~as b ->
    Equal[.A, a, b] ->
    (A -> *) ~as prop ->
    prop[a] ->
    prop[b]
    A => a => b => e => prop => x =>
    ~match e {
        refl(value) = x
        : prop[b]
    }