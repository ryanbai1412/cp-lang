// expect: E0302
// rule: std.persistent.PersistentSegTree
// A `Version` names the structure type that made it, so a `PersistentArray` version can't be read through a `PersistentSegTree` (decision 519).
use std.persistent.{PersistentArray, PersistentSegTree}
domain N = 0..3
let t = PersistentSegTree<N, Int>(+, fill: 0)
let p = PersistentArray<N, Int>(fill: 0)
print(t.all(p.base)) // <- here
