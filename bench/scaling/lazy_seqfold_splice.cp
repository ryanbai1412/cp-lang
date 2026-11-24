use std.lazy.{SeqFold}
use std.algebra.{Action, add}
let n: Int = read()
var s = 0

let xs = [i * 7919 % n for i in 0..n]
let act = Action<Int, Int>.of(add, +)
let k = n / 4
let ys = [i for i in 0..k]
for _ in 0..80:
    var c = SeqFold<Int, Int>(+, xs, act: act)
    c.insert_many(n / 3, ys) // timed: lazy.SeqFold.insert_many
    c.erase_range(n / 5..n / 5 + k) // timed: lazy.SeqFold.erase_range
    var d = c.split_at(n / 2) // timed: lazy.SeqFold.split_at [let: k = n / 2]
    c.splice(n / 7, mut d) // timed: lazy.SeqFold.splice
    s += c.len
print(s)
