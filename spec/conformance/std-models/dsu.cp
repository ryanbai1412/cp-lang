// expect: ok, output "ok"
// input: "1 300\n"
// model: Dsu
// model: DsuOf
// Dsu<V> and DsuOf<V, Int> (a join that does not commute) against a root-per-position list:
// seeded random unite, merge, set, mark and undo_to, with find, same, size, components, roots,
// values and groups compared after each op. Reads `seed n`; prints the first op that differs.
use std.dsu.{Mark}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..7
fn join(x: Int, y: Int) -> Int = (x * 7 + y) % 1_000_003
var rng = Rng(seed0)
var d = Dsu<V>(rollback: rng.int(0, 1) == 1)
var o = DsuOf<V, Int>(init: v => v.ord + 1, join: join)
var root = [i for i in 0..7]
var val = [i + 1 for i in 0..7]
var marks: List<(Mark, Mark, List<Int>, List<Int>)> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 5)
    let (a, b) = (rng.int(0, 6), rng.int(0, 6))
    let (ra, rb) = (root[a], root[b])
    let sa = root.count(r => r == ra)
    let sb = root.count(r => r == rb)
    let (kept, gone) = if sb > sa: (rb, ra) else: (ra, rb)
    if op <= 1:
        let want = if ra == rb: None else: Some((kept: V(kept), merged: V(gone)))
        let got = if op == 0: d.merge(V(a), V(b)) else: (if d.unite(V(a), V(b)): want else: None)
        let got_of = if op == 0: o.merge(V(a), V(b)) else: (if o.unite(V(a), V(b)): want else: None)
        if got != want or got_of != want: bad = "op {k}: merge {a} {b}"
        if ra != rb:
            let joined = join(val[ra], val[rb])
            for i in 0..7:
                if root[i] == gone: root[i] = kept
            val[kept] = joined
    elif op == 2:
        o.set(V(a), b * 10)
        val[ra] = b * 10
    elif op == 3:
        marks.push((d.mark(), o.mark(), root, val))
    elif op == 4 and marks.len > 0:
        let i = rng.int(0, marks.len - 1)
        let (md, mo, r0, v0) = marks[i]
        d.undo_to(md)
        o.undo_to(mo)
        root = r0
        val = v0
        while marks.len > i + 1: marks.pop()
    else:
        if d.size(V(a)) != root.count(r => r == ra) or o.size(V(a)) != root.count(r => r == ra):
            bad = "op {k}: size {a}"
    if bad != "": break
    let roots = [V(i) for i in 0..7 if root[i] == i]
    var groups: List<List<V>> = []
    for r in roots:
        groups.push([V(i) for i in 0..7 if root[i] == r.ord])
    groups.sort(by: g => g[0])
    for i in 0..7:
        if d.find(V(i)) != V(root[i]) or o.find(V(i)) != V(root[i]) or o.get(V(i)) != val[root[i]]:
            bad = "op {k}: find or get {i}"
        for j in 0..7:
            if d.same(V(i), V(j)) != (root[i] == root[j]) or o.same(V(i), V(j)) != (root[i] == root[j]):
                bad = "op {k}: same {i} {j}"
    if bad == "" and (d.components != roots.len or o.components != roots.len):
        bad = "op {k}: components {d.components} {o.components} want {roots.len}"
    elif bad == "" and (d.roots().collect() != roots or o.roots().collect() != roots):
        bad = "op {k}: roots"
    elif bad == "" and o.values.collect() != [val[r.ord] for r in roots]:
        bad = "op {k}: values"
    elif bad == "" and (d.groups() != groups or o.groups() != groups):
        bad = "op {k}: groups {d.groups()} want {groups}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
