// expect: ok, output "ok"
// input: "1 300\n"
// model: Centroids
// Centroid decompositions of seeded random trees (one per op) against breadth-first search: a
// random centroid's piece is what it reaches avoiding shallower centroids, with distances and
// branches, the centroid splits its piece in halves, and parent, depth and ancestors agree with
// the pieces. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.tree.{Centroids}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..11
var rng = Rng(seed0)
fn reach(adj: [V]List<V>, s: V, ok: V -> Bool) -> (List<V>, [V]Int, [V]V):
    var d: [V]Int = -1
    var br: [V]V = [V] _ => s
    d[s] = 0
    var q = [s]
    var i = 0
    while i < q.len:
        let x = q[i]
        i += 1
        for y in adj[x]:
            if d[y] < 0 and ok(y):
                d[y] = d[x] + 1
                br[y] = if x == s: y else: br[x]
                q.push(y)
    (q, d, br)
var bad = ""
for k in 0..n:
    var adj: [V]List<V> = []
    var perm = [V(i) for i in 0..11]
    rng.shuffle(mut perm)
    for i in 1..11:
        let p = perm[rng.int(0, i - 1)]
        adj[perm[i]].push(p)
        adj[p].push(perm[i])
    let cd = Centroids(adj)
    let c = V(rng.int(0, 10))
    let dc = cd.depth(c)
    let (q, d, br) = reach(adj, c, y => cd.depth(y) > dc)
    var got = cd.piece(c).collect()
    var want = [(v: y, dist: d[y], branch: br[y]) for y in q]
    got.sort()
    want.sort()
    let halves = adj[c].all(y => d[y] < 0 or q.count(z => br[z] == y) * 2 <= q.len)
    let owner = [x for x in V if cd.depth(x) == dc - 1 and reach(adj, x, y => cd.depth(y) >= dc).0.contains(c)]
    let v = V(rng.int(0, 10))
    let anc = [(c: x, dist: reach(adj, x, y => cd.depth(y) >= cd.depth(x)).1[v]) for x in V if reach(adj, x, y => cd.depth(y) >= cd.depth(x)).0.contains(v)]
    let sorted_anc = [e for dd in (0..12).rev, e in anc if cd.depth(e.c) == dd]
    if cd.depth(cd.root) != 0 or cd.parent(cd.root) != None: bad = "op {k}: root"
    elif got != want: bad = "op {k}: piece {c}"
    elif not halves: bad = "op {k}: {c} is no centroid of its piece"
    elif cd.parent(c) != (if dc == 0: None else: owner.first): bad = "op {k}: parent {c}"
    elif cd.ancestors(v).collect() != sorted_anc: bad = "op {k}: ancestors {v}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
