// CSES 1137 Subtree Queries: change a vertex value, sum the values in a subtree (root 1).
// In: n q, v1..vn, n-1 edges "a b", then q queries "1 s x" (v_s = x) or "2 s". Out: each subtree sum.
domain V = 1..=read<1..=2e5>()
let q = read<1..=2e5>()
let value = read<[V]1..=1e9>()
let t = Rooted(Graph.undirected(read<[V.count - 1](u: V, v: V)>()), root: 1)
var sums = Fenwick(t.reorder(value))

enum Query:
    @tag(1) Set(s: V, x: 1..=1e9)
    @tag(2) Sum(s: V)

for _ in 0..q:
    match read<Query>():
        Set(s, x) => sums.set(t.pos(s), x)
        Sum(s) => print(sums.fold(t.sub(s)))
