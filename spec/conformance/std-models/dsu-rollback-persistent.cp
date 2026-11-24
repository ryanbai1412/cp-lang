// expect: ok, output "ok"
// input: "1 300\n"
// model: RollbackArray
// model: PersistentDsu
// RollbackArray<D, Int> against a List with saved copies: seeded random set, mark and undo_to.
// PersistentDsu<V> against a root-per-position list kept for every version: seeded random unite
// on a random earlier version, with find, same and size compared on the new and the old version.
// Reads `seed n`; prints the first op that differs.
use std.dsu.{DsuNode, Mark, PersistentDsu, RollbackArray}
use std.persistent.{PArena}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain D = 0..6
domain V = 0..7
var rng = Rng(seed0)
var arr = RollbackArray<D, Int>(init: i => i.ord * 3)
var a = [i * 3 for i in 0..6]
var marks: List<(Mark, List<Int>)> = []
var ar = PArena<DsuNode<V>>()
var vers = [PersistentDsu<V>.new(mut ar)]
var roots = [[i for i in 0..7]]
var bad = ""
for k in 0..n:
    let op = rng.int(0, 3)
    if op == 0:
        let (i, x) = (rng.int(0, 5), rng.int(-9, 9))
        arr.set(D(i), x)
        a[i] = x
    elif op == 1:
        marks.push((arr.mark(), a))
    elif op == 2 and marks.len > 0:
        let i = rng.int(0, marks.len - 1)
        arr.undo_to(marks[i].0)
        a = marks[i].1
        while marks.len > i + 1: marks.pop()
    else:
        let src = rng.int(0, vers.len - 1)
        let (x, y) = (rng.int(0, 6), rng.int(0, 6))
        var root = roots[src]
        let (rx, ry) = (root[x], root[y])
        let before = ar.len
        vers.push(vers[src].unite(mut ar, V(x), V(y)))
        if rx == ry and ar.len != before: bad = "op {k}: unite inside a set allocated"
        if rx != ry:
            let (sx, sy) = (root.count(r => r == rx), root.count(r => r == ry))
            let (kept, gone) = if sy > sx: (ry, rx) else: (rx, ry)
            for i in 0..7:
                if root[i] == gone: root[i] = kept
        roots.push(root)
        for w in [src, vers.len - 1]:
            for i in 0..7:
                if vers[w].find(ar, V(i)) != V(roots[w][i]) or vers[w].size(ar, V(i)) != roots[w].count(r => r == roots[w][i]):
                    bad = "op {k}: version {w} find or size {i}"
                for j in 0..7:
                    if vers[w].same(ar, V(i), V(j)) != (roots[w][i] == roots[w][j]): bad = "op {k}: version {w} same {i} {j}"
    for i in 0..6:
        if arr.get(D(i)) != a[i]: bad = "op {k}: get {i} is {arr.get(D(i))} want {a[i]}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
