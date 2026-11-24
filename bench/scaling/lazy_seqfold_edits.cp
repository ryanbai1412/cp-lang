use std.lazy.{SeqFold}
use std.algebra.{Action, add}
let n: Int = read()
var s = 0

let xs = [i * 7919 % n for i in 0..n]
let act = Action<Int, Int>.of(add, +)
var sf = SeqFold<Int, Int>(+, xs, act: act)
// 3n / 2 inserts take the sequence from n to 2.5n at every size
for i in 0..n / 2:
    sf.push(i) // timed: lazy.SeqFold.push
    sf.push(i) // timed: lazy.SeqFold.push
    sf.insert_at(i * 7919 % n, i) // timed: lazy.SeqFold.insert_at
print(s)
