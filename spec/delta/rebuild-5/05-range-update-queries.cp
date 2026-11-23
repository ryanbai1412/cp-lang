// CSES 1651 Range Update Queries: add u to x[a..=b], or print x[k].
// Input: n q, the array, then q lines "1 a b u" or "2 k". Output: one line per type-2 query.
let (n, q) = read<(1..=2e5, 1..=2e5)>()
domain V = 1..=n
var st = SegTree(a: read<[V]1..=1e9>(), act: add)

enum Query:
    @tag(1) Add(at: Range<V>, u: 1..=1e9)
    @tag(2) Ask(k: V)

repeat q:
    match read<Query>():
        Add(at, u) => st.apply(at, u)
        Ask(k) => print(st[k])
