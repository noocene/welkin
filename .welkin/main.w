dup:
* ~as A  |-> 
'(A -> A) -> 
'(A -> A)

A ||>
func |>
func < func
> arg |> func(func(arg))

old_main:
'Bool

id < dup[Bool](> Bool::not)
val <> id(Bool::false)
> Bool::not(val)

test:
Vector[Bool, Nat::one]

Vector::cons[Bool, Nat::zero](Bool::true, Vector::nil[Bool])

main:
Bool

Vector::head[Bool, Nat::zero](test)