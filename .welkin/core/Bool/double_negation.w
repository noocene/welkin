double_negation:
Bool ~as b ->
Equal[Bool, Bool::not(Bool::not(b)), b]

b |>
~match b {
	true  = Equal::refl[Bool, Bool::true]
    false = Equal::refl[Bool, Bool::false]
    : b |> Equal[Bool, Bool::not(Bool::not(b)), b]
}