// COCI 2012/13 #1 Ljubomora: every marble goes to one of n children, and a
// child gets marbles of one colour only. Make the largest handful as small
// as possible.
// original: corpus/submissions/5245322.cpp
use std.math.{ceil_div}                                             // WISH F1

let kids: 1..=1e9 = read()
let m: 1..=min(3e5, kids) = read()
let marbles: [m]1..=1e9 = read()

// with handfuls of at most x, a colour of c marbles takes ceil(c / x) children
print((1..=1e9).first_where(x => sum(ceil_div(c, x) for c in marbles) <= kids)!)     // WISH R1
