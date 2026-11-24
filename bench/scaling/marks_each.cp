// The walks live apart from marks.cp so their tables don't crowd its
// working set out of cache.
use std.marks.{Marks, MarkMap, Interner, Arena}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0

var m = Marks<V>()
var mm = MarkMap<V, Int>(0)
var t = Interner<Int>()
var ab = Arena<(Int, Int)>()
for i in 0..n:
    m.add(V(i))
    mm.set(V(i), i)
    s += t.id(i * 7919)
    ab.alloc((i, i))
for _ in 0..200:
    s += m.each().collect().len // timed: marks.Marks.each
    s += mm.items.collect().len // timed: marks.MarkMap.items
// Interner.each and Arena.items cost O(1) a step: take one step a call.
for _ in 0..reps:
    for k in t.each(): // timed: marks.Interner.each
        s += k
        break
    for (id, x) in ab.items: // timed: marks.Arena.items
        s += x.0
        break
print(s)
