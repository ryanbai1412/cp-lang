use std.dsu.{Dsu}
use std.graph.{bfs}
let n: 2..=8 = read()
domain V = 1..=n
let es: [n - 1](u: V, v: V, len: 1..=9) = read()
var d = Dsu<V>.new()
var joined = 0
for e in sorted(es, by: .len):
    if d.unite(e.u, e.v): joined += 1
print(joined, d.parts, yes(d.same(V(1), V.last!)), d.size(V(1)), yes(d.find(V(2)) == d.find(V(1))))
var adj: [V]List<V> = []
for (u, v, len) in es:
    adj[u].push(v)
    adj[v].push(u)
let hops = bfs(adj, from: [V.first!])
print([hops[v]! for v in V])
let tree = Rooted(Graph.undirected(es).weighted(a => a.len), root: V.first!)
print([tree.dist(tree.root, v) for v in V], tree.depth(V.last!), tree.children(V.first!).collect())
let plain = Rooted(adj, root: V.first!)
print(plain.dist(V.first!, V.last!), plain.lca(V(2), V(3)))
let wanted = Set([(e.u.ord, e.len) for e in es])
print(yes((0, 3) in wanted), yes((5, 5) in wanted))
// a step function: add a count over a range, then walk the stretches of equal counts
domain X = 0..=10
var cover: [X]Int = 0
for (lo, hi, by) in [(2, 5, 1), (4, 9, 2), (-3, -1, 7)]:
    for x in X:
        if lo <= x.ord and x.ord <= hi: cover[x] += by
for r in cover.runs(): print(r.keys.first!, r.len, r[r.keys.first!])
// a scan's table holds the running value before each cut; `prefix(k)` is the fold of the first k
fn running<M>(xs: List<Int>, f: M) -> List<Int>:
    let p = xs.scan(f)
    [p.prefix(k) for k in 1..=xs.len]
print(running([3, 1, 4, 1, 5], +), running([1, 2, 3], +), running([3, 1, 4, 1, 5], max), running([4, 6, 9], gcd))
var lists: [V]List<Int> = []
lists[V(1)].push(5)
var mine = lists[V(1)].take()
mine.push(6)
var other = [1]
swap(mut mine, mut other)
print(mine, other, lists[V(1)].len)
print((1..=100).find(x => x * x >= 50) ?? -1, (1..=5).find(x => x > 9) ?? -1, V.find(v => v.ord >= 1)!)
let vs: [V]V = [V] v => v
print([w.len for w in vs.windows(3)].len, sum(w.first!.ord for w in vs.windows(2)))
print(V.wrap(Int(V.first!) - 1), V.wrap(Int(V.last!) + 2), V.wrap(Int(V(2)) + n))
let cnt = ([V] v => if v.ord % 2 == 0: 1 else: 0).scan(+)
print(cnt.all, cnt.fold(V(3)..=V.last!))
fn f(v: V) -> ?Int: if v.ord % 3 == 0: None else: v.ord
var seg = SegTree<V, ?Int>.of(min, f)
print(seg.all ?? -1, seg.fold(V(1)..=V(1)) ?? -1)
var sums = SegTree<V, Int>.of(+, v => v.ord)
sums.refresh(V(2))
print(sums.all)
