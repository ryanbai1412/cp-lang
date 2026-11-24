// A view made from variables points to them (R05.4.4): `ms.within(mar)`
// lends both the multiset and its arena, copying neither.
use std.persistent.{PArena, PMultiset}
let n: Int = read()
domain V = 0..n
var mar = PArena<PMultiset<V>>()
var ms = PMultiset<V>.empty()
for i in 0..n: ms = ms.with(mut mar, V(i % 3))
let w = ms.within(mar)
print(w.len, yes(w.is_empty))
var total = 0
for _ in 0..n:
    let v = ms.within(mar)
    total += v.len
print(total)
