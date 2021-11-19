~data WhelkRequest {
    print(data: Sized[String]),
    prompt(data: Unit),
    define(
        name: Sized[String],
        type: Ast,
        term: Ast
    ),
    loop[
        state: *,
        steps: Size,
        complexity: Size
    ](
        initial: state,
        continue: state -> Bool,
        step: BoxPoly[
            state,
            complexity
        ] -> BoxPoly[
            WhelkIO[state, steps],
            complexity
        ]
    )
}