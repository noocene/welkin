Size:
*

(
    Size -> *
) ~as prop     |-self->
'(
    Size ~as n |-> 
    prop(n)     -> 
    prop(Size::succ(n))
)                    ->
'(
    prop(Size::zero) ->
    prop(self)
)