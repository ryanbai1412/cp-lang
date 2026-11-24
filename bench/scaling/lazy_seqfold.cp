use std.lazy.{SeqFold}
use std.algebra.{Action, add}
let n: Int = read()
let reps = 1 << 17
var s = 0

let xs = [i * 7919 % n for i in 0..n]
let act = Action<Int, Int>.of(add, +)
for _ in 0..8:
    s += SeqFold<Int, Int>(+, xs, act: act).total // timed: lazy.SeqFold.new
var sf = SeqFold<Int, Int>(+, xs, act: act)
for i in 0..reps:
    let p = i * 7919 % n
    s += sf.len // timed: lazy.SeqFold.len
    if sf.is_empty: s += 1 // timed: lazy.SeqFold.is_empty
    s += sf.total // timed: lazy.SeqFold.total
    s += sf.get(i * 13 % n) // timed: lazy.SeqFold.get
    s += sf.fold(p / 2..p) // timed: lazy.SeqFold.fold
for _ in 0..40:
    for x in sf.items: // timed: lazy.SeqFold.items
        s += x // timed: lazy.SeqFold.items
print(s)
