// CSES 1651 Range Update Queries: range add, point query.
// Input: n q, then x1..xn, then q queries "1 a b u" (add u to x_a..x_b) or "2 k". Output: x_k per "2 k".
enum Query:
    @tag(1) Add(r: Range<V>, u: 1..=1e9)
    @tag(2) Get(k: V)

domain V = 1..=read<1..=2e5>()
let q = read<1..=2e5>()
var st = SegTree(a: read<[V]1..=1e9>(), act: add)
repeat q:
    match read<Query>():
        Add(r, u) => st.apply(r, u)
        Get(k) => print(st[k])
