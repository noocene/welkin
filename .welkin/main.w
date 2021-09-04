main:
'Sized[String]

String::concat_sized(
    > Sized::new[String](
        ~literal Size 6,
        ~literal String "Hello "
    ),
    > Sized::new[String](
        ~literal Size 6,
        ~literal String "world!"
    )
)