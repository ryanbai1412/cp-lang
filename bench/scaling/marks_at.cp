// Arena.at lends the node in place: one index, whatever the arena's size.
use std.marks.{Arena}
let n: Int = read()
var s = 0
var a = Arena<Int>()
let ids = [a.alloc(i) for i in 0..n]
for i in 0..64 * n:
    a.at(ids[i * 7919 % n]) += 1 // timed: marks.Arena.at
    s += a.get(ids[i * 13 % n])
print(s)
