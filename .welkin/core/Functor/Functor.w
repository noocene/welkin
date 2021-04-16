~data Functor (F: * -> *) {
    new(map: * ~as A -> * ~as B -> (A -> B) -> F(A) -> F(B))
}