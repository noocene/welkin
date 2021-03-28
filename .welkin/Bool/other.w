
binder : A -> a : A -> List[A] = A a >
    List[A]::cons[
        a,
        List[A]::nil
    ]

binder : A : * -> 
         a : A -> 
         List[A] 
    = *

data List a {
    nil,
    cons(a, List[a])
}