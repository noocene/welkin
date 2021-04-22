~data Delay A {
    now(value: A),
    later(thunk: Thunk[Delay[A]])
}