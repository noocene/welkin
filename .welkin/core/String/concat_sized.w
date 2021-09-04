concat_sized:
'Sized[String] ->
'Sized[String] ->
'Sized[String]

a |>
b |>
a < a
b < b
(~match a {
	new(az, as) = ~match b {
		new(bz, bs) =
			string < String::concat(az, bz, > as, > bs)
			> Sized::new[String](Size::add(az, bz), string)
		: _ |> 'Sized[String]
	}
	: _ |> 'Sized[String]
})