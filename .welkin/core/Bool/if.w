if: 
*    ~as A   |-> 
*    ~as B   |->
Bool ~as bool -> 
A             -> 
B             ->
~match bool {
    true  = A
    false = B
    : *
}

A ||> B ||> 
bool |> 
a |> b |>
~match bool {
    true = a
    : A

    false = b
    : B
}
