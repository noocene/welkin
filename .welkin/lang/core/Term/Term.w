~data Index {
    new(value: Nat)
}

~data Term A {
    variable(index: Index),
    lambda(
        body: Term[A], 
        erased: Bool
    ),
    apply(
        applied: Term[A],
        argument: Term[A],
        erased: Bool
    ),
    put(term: Term[A]),
    duplicate(
        expression: Term[A],
        body: Term[A]
    ),
    reference(value: A),

    universe,
    function(
        argument_type: Term[A],
        return_type: Term[A],
        erased: Bool
    ),
    annotation(
        checked: Bool,
        expression: Term[A],
        type: Term[A]
    )
}