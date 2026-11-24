// expect: ok, output "3 6 3 0\n2 7 3 0"
// rule: std.persistent.PersistentSegTree
// input: "2\n3 1 2 3\n2 4 3\n"
// A tree declared inside a test case is fresh in every case: its nodes and versions live in the
// value, so case two's tree holds only its own nodes and versions.
use std.persistent.{PersistentArray, PersistentSegTree}
domain K = 0..5
@!multitest
let n: 1..=5 = read()
let xs: [0..n]Int = read()
var t = PersistentSegTree<K, Int>(+, fill: 0)
var p = PersistentArray<K, Int>(fill: 0)
var v = t.base
var w = p.base
for i in 0..n:
    v = t.update(v, K(i), x => x + xs[i])
    w = p.set(w, K(i), xs[i])
print(n, t.all(v), p.get(w, K(n - 1)), t.all(t.base))
