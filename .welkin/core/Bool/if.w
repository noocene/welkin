if: 
*    ~as A   |-> 
*    ~as B   |->
Bool ~as bool -> 
A             -> 
B             ->
~match bool {
    true  = A
    false = B
    : _ |> *
}

A ||> B ||> 
bool |> 
a |> b |>
~match bool {
    true = a
    : _ |> A

    false = b
    : _ |> B
}
