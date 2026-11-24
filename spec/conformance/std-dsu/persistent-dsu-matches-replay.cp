// expect: ok, output "YES"
// rule: std.dsu.PersistentDsu
// Random `unite`s branching from random earlier versions, checked against a plain `Dsu` that
// replays each version's unions from scratch: `same`, `size` and `find`'s set agree, and a
// `unite` inside one set returns the same version without allocating.
use std.dsu.{DsuNode, PersistentDsu}
use std.persistent.{PArena}
use std.rand.{Rng}
domain V = 0..9
var rng = Rng(3)
var ar = PArena<DsuNode<V>>()
var vers = [PersistentDsu<V>.new(mut ar)]
// Version `i` came from `ops[i].0` by uniting `ops[i].1` and `ops[i].2`.
var ops: List<(Int, Int, Int)> = [(-1, 0, 0)]
var ok = ar.len == 2 * 9 - 1
for _ in 0..120:
    let src = rng.int(0, vers.len - 1)
    let (a, b) = (rng.int(0, 8), rng.int(0, 8))
    let before = ar.len
    let same = vers[src].same(ar, V(a), V(b))
    vers.push(vers[src].unite(mut ar, V(a), V(b)))
    if same and ar.len != before: ok = false
    ops.push((src, a, b))
for i in 0..vers.len:
    var plain = Dsu<V>()
    var j = i
    while j > 0:
        let (up, a, b) = ops[j]
        plain.unite(V(a), V(b))
        j = up
    for a in V:
        if vers[i].size(ar, a) != plain.size(a): ok = false
        if not plain.same(vers[i].find(ar, a), a): ok = false
        for b in V:
            if vers[i].same(ar, a, b) != plain.same(a, b): ok = false
print(yes(ok))
