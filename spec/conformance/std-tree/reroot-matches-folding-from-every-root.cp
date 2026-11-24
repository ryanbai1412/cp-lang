// expect: ok, output "YES"
// `Rooted.reroot` gives every vertex the fold of the whole tree rooted there: its parent's side
// first, then its children in child order, matching a direct recursion per root with a
// non-commuting `op`.
use std.rand.Rng

domain V = 0..12
var rng = Rng(11)
fn r(mut g: Rng, m: Int) -> Int = Int(g.next() % U64(m))
let mo = 1_000_003
let aff = Monoid<(Int, Int)>(id: (1, 0), op: (x, y) => (x.0 * y.0 % mo, (x.1 * y.0 + y.1) % mo))
fn lift(x: (Int, Int), c: V, p: V) -> (Int, Int) = (x.0, (3 * x.1 + c.ord + 2 * p.ord) % mo)
fn finish(s: (Int, Int), v: V) -> (Int, Int) = (s.0 * (v.ord + 2) % mo, (s.1 + v.ord + 1) % mo)
var ok = true
for _ in 0..50:
    var adj: [V]List<V> = []
    for i in 1..12:
        let j = r(mut rng, i)
        adj[V(i)].push(V(j))
        adj[V(j)].push(V(i))
    let t = Rooted(adj, root: V(r(mut rng, 12)))
    fn val(v: V, from: ?V) -> (Int, Int):
        var acc = aff.id
        if let pv = t.par[v], Some(pv) != from: acc = aff.op(acc, lift(val(pv, Some(v)), pv, v))
        for c in t.children(v):
            if Some(c) != from: acc = aff.op(acc, lift(val(c, Some(v)), c, v))
        finish(acc, v)
    let got = t.reroot(aff, lift:, finish:)
    for v in V: ok = ok and got[v] == val(v, None)
print(if ok: "YES" else: "NO")
