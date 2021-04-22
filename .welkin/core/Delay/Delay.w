~data Delay A {
    now(value: A),
    later(value: Thunk[Delay[A]])
}