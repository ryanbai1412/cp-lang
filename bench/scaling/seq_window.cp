use std.hashmap.{Counter}
let n: Int = read()
let zs = [i % 3 for i in 0..n]
var s = 0
for r in 0..16:
    s += zs.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 2).len // timed: seq.maximal
    s += zs.ring.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 2).len // timed: seq.Ring.maximal
print(s)
