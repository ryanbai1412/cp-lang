// expect: ok, output "0 5 2\n0 5"
// A persistent structure's versions share nodes held by the value the program declares, not a hidden module arena: a second tree has its own nodes and its own versions (decision 519).
use std.persistent.{PersistentSegTree}
domain N = 0..3
let z: [N]Int = [0, 0, 0]
var t = PersistentSegTree<N, Int>(+, a: z)
var u = PersistentSegTree<N, Int>(+, a: z)
let v1 = t.set(t.base, N(1), 5)
let w1 = u.set(u.base, N(2), 2)
print(t.all(t.base), t.all(v1), u.all(w1))
print(u.all(u.base), t.fold(v1, N(1)..=N(2)))
