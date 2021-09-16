slow_sub_zero_r_elim:
Size ~as n |->
Equal[
    'Size,
    Size::slow_sub(> n, Size::zero),
    > n
]

n ||>
Equal::refl['Size, > n]