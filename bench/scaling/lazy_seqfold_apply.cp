use std.lazy.{SeqFold}
use std.algebra.{Action, add}
let n: Int = read()
let reps = 1 << 17
var s = 0

let xs = [i * 7919 % n for i in 0..n]
let act = Action<Int, Int>.of(add, +)
var sf = SeqFold<Int, Int>(+, xs, act: act)
for i in 0..reps:
    let p = i * 7919 % n
    sf.apply(p / 2..p, 1) // timed: lazy.SeqFold.apply
    sf.reverse(p / 3..p) // timed: lazy.SeqFold.reverse
print(s)
