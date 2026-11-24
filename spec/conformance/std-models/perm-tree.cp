// expect: ok, output "ok"
// input: "1 300\n"
// model: PermTree
// Permutation trees of seeded random permutations (one per op) against the permutation itself:
// every node from the root down has a run of positions split by its children, values inside
// vals, an interval of values unless Partial, and Incr/Decr children whose values meet.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.tree.{PermKind, PermTree}
let seed0 = read<Int>()
let n = read<Int>()
domain I = 0..10
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    var vs = [I(i) for i in 0..10]
    rng.shuffle(mut vs)
    let p: [I]I = [I] i => vs[i.ord]
    let pt = PermTree(p)
    var todo = [pt.root]
    var leaves = 0
    while let x = todo.pop():
        let (s0, s1) = pt.span(x).ends
        let (v0, v1) = pt.vals(x).ends
        let ys = [p[I(i)].ord for i in s0..s1]
        if ys.min()! != v0 or ys.max()! + 1 != v1: bad = "op {k}: vals {s0}..{s1}"
        elif pt.kind[x] != PermKind.Partial and v1 - v0 != s1 - s0: bad = "op {k}: not an interval {s0}..{s1}"
        match pt.node(x):
            Leaf(i):
                leaves += 1
                if s1 - s0 != 1 or i.ord != s0 or pt.leaf(i) != x or pt.kind[x] != PermKind.Leaf: bad = "op {k}: leaf {s0}"
            Inner(l, r, kind):
                let (l0, l1) = pt.span(l).ends
                let (r0, r1) = pt.span(r).ends
                if pt.left(x) != l or pt.right(x) != r or kind != pt.kind[x]: bad = "op {k}: children {s0}..{s1}"
                elif l0 != s0 or l1 != r0 or r1 != s1: bad = "op {k}: split {s0}..{s1}"
                elif kind == PermKind.Incr and pt.vals(l).ends.1 != pt.vals(r).ends.0: bad = "op {k}: incr {s0}..{s1}"
                elif kind == PermKind.Decr and pt.vals(r).ends.1 != pt.vals(l).ends.0: bad = "op {k}: decr {s0}..{s1}"
                todo.push(l)
                todo.push(r)
        if bad != "": break
    if bad == "" and leaves != 10: bad = "op {k}: {leaves} leaves"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
