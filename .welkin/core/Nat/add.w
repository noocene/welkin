// add:
// Nat |-> 
// Nat |->
// Nat

// a ||> b ||>
// ~match a {
//    zero       = b
//    succ(pred) = Nat::succ(Nat::add[pred, b])
//    : Nat
// }