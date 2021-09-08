main:
'Sized[String]

string < String::concat(
    ~literal Size 3,
    ~literal Size 2,
    > ~literal String "Hel",
    > ~literal String "lo"
)
> Sized::new[String](
    ~literal Size 5,
    string
)