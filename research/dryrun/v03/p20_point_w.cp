// CSES 2189 Point Location Test: is p3 left of, right of, or on the line p1 -> p2?
use std.geometry.turn
@!multitest
let (p1, p2, p3) = read<(Pt, Pt, Pt)>()
let side = match turn(p1, p2, p3):
    1 => "LEFT"
    -1 => "RIGHT"
    _ => "TOUCH"
print(side)
