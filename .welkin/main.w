dup : * ~as A -> '(A -> A) -> '(A -> A)
    A => func =>
    func < func
    > arg => func[func[arg]]

wrap : * ~as A -> A -> 'A
    A => value =>
    > value

false : Bool
    Bool::false
true : Bool
    Bool::true

not : Bool -> Bool
    ~core {
        \x ([x \self Bool] false true)
    }

main : 'Bool
    id < (dup[Bool])[> not]
    val <
        > id[Bool::false]
    > not[val]