dup : * ~as A -> '(A -> A) -> '(A -> A)
    A => func =>
    func < func
    > arg => func[func[arg]]

main : 'Bool
    id < (dup[Bool])[> Bool::not]
    val <
        > id[Bool::false]
    > Bool::not[val]