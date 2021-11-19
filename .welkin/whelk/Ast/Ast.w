~data Ast {
    Lambda(
        erased: Bool,
        body: Ast
    ),
    Variable(index: Size),
    Application(
        erased: Bool,
        function: Ast,
        argument: Ast
    ),
    Put(term: Ast),
    Duplication(
        expression: Ast,
        body: Ast
    ),
    Reference(name: Sized[String]),

    Universe,
    Function(
        erased: Bool,
        argument_type: Ast,
        return_type: Ast
    ),
    Wrap(term: Ast)
}