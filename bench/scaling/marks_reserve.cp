// n: 512
// reserve moves the nodes (Arena) or rehashes the keys (Interner). At n = 512 the clone
// and the doubled block stay under 128 KB at 4n too, so malloc serves both from the heap
// and no size pays fresh-page faults that the other doesn't.
use std.marks.{Arena, Interner}
let n: Int = read()
var s = 0
var ab = Arena<(Int, Int)>()
for i in 0..n: ab.alloc((i, i))
for _ in 0..40000:
    var b = ab.clone()
    b.reserve(2 * n) // timed: marks.Arena.reserve
    s += b.len
var ub = Interner<(Int, Int)>()
for i in 0..n: ub.id((i, i))
for _ in 0..8000:
    var u = ub.clone()
    u.reserve(2 * n) // timed: marks.Interner.reserve
    s += u.len
print(s)
