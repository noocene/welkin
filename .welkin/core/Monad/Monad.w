~data Monad (M: * -> *) {
    new(
        bind: * ~as A -> * ~as B -> M[A] -> (A -> M[B]) -> M[B],
        pure: * ~as A -> A -> M[A]
    )
}