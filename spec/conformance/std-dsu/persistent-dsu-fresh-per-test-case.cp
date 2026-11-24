// expect: ok, output "25 YES 3\n13 NO 1"
// rule: std.dsu.PersistentDsu
// input: "2\n6 2\n0 1\n1 2\n4 1\n2 3\n"
// An arena declared inside a test case is fresh in every case: the library keeps no pool of its
// own, so case two's arena holds its 7 base nodes and 6 path copies, not 25 + 13.
use std.dsu.{DsuNode, PersistentDsu}
use std.persistent.{PArena}
@!multitest
let n: 2..=10 = read()
domain V = 0..n
let m: 0..=5 = read()
var ar = PArena<DsuNode<V>>()
let d = PersistentDsu<V>.new(mut ar)
var e = d
for _ in 0..m:
    let a: V = read()
    let b: V = read()
    e = e.unite(mut ar, a, b)
print(ar.len, yes(e.same(ar, V(0), V(1))), e.size(ar, V(1)))
