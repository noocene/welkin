functor:
Size ~as length |->
Functor[A |> Vector[A, length]]

length ||>
Functor::new[A |> Vector[A, length]](A ||> B ||> Vector::map[A, B, length])