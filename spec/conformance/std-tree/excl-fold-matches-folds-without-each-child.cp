// expect: ok, output "YES"
// `Rooted.excl_fold` gives, for each child in child order, the fold of every other child's value in
// child order, with a non-commuting `op`.
use std.rand.Rng

domain V = 0..12
var rng = Rng(7)
fn r(mut g: Rng, m: Int) -> Int = Int(g.next() % U64(m))
let p = 1_000_003
let aff = Monoid<(Int, Int)>(id: (1, 0), op: (x, y) => (x.0 * y.0 % p, (x.1 * y.0 + y.1) % p))
var ok = true
for _ in 0..100:
    var adj: [V]List<V> = []
    for i in 1..12:
        let j = r(mut rng, i)
        adj[V(i)].push(V(j))
        adj[V(j)].push(V(i))
    let t = Rooted(adj, root: V(r(mut rng, 12)))
    for v in V:
        let cs = t.children(v).collect()
        let got = t.excl_fold(v, aff, c => (c.ord + 2, 3 * c.ord + 1)).collect()
        ok = ok and got.len == cs.len
        for i in 0..cs.len:
            var want = aff.id
            for j in 0..cs.len:
                if j != i: want = aff.op(want, (cs[j].ord + 2, 3 * cs[j].ord + 1))
            ok = ok and got[i].c == cs[i] and got[i].rest == want
        let sums = t.excl_fold(v, +, c => t.size(c)).collect()
        for i in 0..cs.len: ok = ok and sums[i].rest == t.size(v) - 1 - t.size(cs[i])
print(if ok: "YES" else: "NO")
