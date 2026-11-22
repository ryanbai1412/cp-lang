// expect: ok, output "0 0 1 4 4"
// rule: 05.10.11
use std.dsu.Dsu

domain V = 0..8
var d = Dsu<V>()
let _ = d.unite(V.nth(0), V.nth(1))
let _ = d.unite(V.nth(2), V.nth(3))
let _ = d.unite(V.nth(0), V.nth(2))
let ro = d
let r1 = ro.find(V.nth(3)).ord
let r2 = ro.find(V.nth(3)).ord
let same = if ro.same(V.nth(1), V.nth(3)): 1 else: 0
let sz = ro.size(V.nth(3))
let g0 = ro.groups()[0].len
print(r1, r2, same, sz, g0)
