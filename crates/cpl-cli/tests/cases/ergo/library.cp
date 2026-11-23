use std.dsu.{Dsu}
use std.graph.{bfs}
use std.seq.{scan}
use std.steps.{Steps}
let n: 2..=8 = read()
domain V = 1..=n
let es: [n - 1](u: V, v: V, len: 1..=9) = read()
var d = Dsu<V>.new()
var joined = 0
for e in sorted(es, by: .len):
    if d.unite(e.u, e.v): joined += 1
print(joined, d.count, d.same(V(1), V.last), d.size(V(1)), d.find(V(2)) == d.find(V(1)))
var adj: [V]List<V> = []
for (u, v, len) in es:
    adj[u].push(v)
    adj[v].push(u)
let hops = bfs(v => adj[v], from: [V.first])
print(hops)
let tree = Rooted(es, V.first, weight: .len)
print([tree.dist(tree.root, v) for v in V], tree.depth(V.last), tree.kids[V.first])
let plain = Rooted(adj, V.first)
print(plain.dist(V.first, V.last), plain.lca(V(2), V(3)))
let wanted = Set([(e.u.ord, e.len) for e in es])
print((0, 3) in wanted, (5, 5) in wanted)
var st = Steps.new()
st.add(2..=5, 1)
st.add(4..=9, 2)
st.add(-3..0, 7)
for (span, count) in st.runs(0..=10): print(span.lo, span.len, count)
print(scan([3, 1, 4, 1, 5], +), scan([1, 2, 3], +), scan([3, 1, 4, 1, 5], max), scan([4, 6, 9], gcd))
var lists: [V]List<Int> = []
lists[V(1)].push(5)
var mine = take(lists[V(1)])
mine.push(6)
var other = [1]
swap(mut mine, mut other)
print(mine, other, lists[V(1)].len)
print((1..=100).first_where(x => x * x >= 50) ?? -1, (1..=5).first_where(x => x > 9) ?? -1, V.all.first_where(v => v.ord >= 1)!)
print([w.len for w in V.windows(3)].len, sum(w.lo.ord for w in V.windows(2)))
print(V.first.cyc(-1), V.last.cyc(2), V(2).cyc(n))
let cnt = prefix([V] v => v.ord % 2 == 0)
print(cnt.sum(V.all), cnt.sum(from: V.last, len: 3))
fn f(v: V) -> ?Int: if v.ord % 3 == 0: None else: v.ord
var seg = SegTree<V, ?Int>.of(min, f)
print(seg.all ?? -1, seg.query(V(1), V(1)) ?? -1)
var sums = SegTree<V, Int>.of(+, v => v.ord)
sums.refresh(V(2))
print(sums.all)
