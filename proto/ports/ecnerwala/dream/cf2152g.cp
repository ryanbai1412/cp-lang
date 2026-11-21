// Codeforces 2152 G: Query Jungle.
// After each subtree flip, find the fewest root paths covering every monster.
// A monster needs its own path iff its next monster in preorder is not in its
// subtree, so a lazy segment tree over preorder maintains `(ends, first, last)`
// for both the active monster set and its complement, swapping on subtree flip.

use std.lazy.{Flip, LazySegTree}
use std.tree.Rooted

@!multitest
domain V = read(1..=250_000)
let monster: [V]Bool = read()
let t = Rooted(Graph<V>.undirected(read<[V.count - 1](V, V)>()), V.first)

struct Side(ends: 0..=V.count, first: ?V, last: ?V):
    const empty = Side(0, none, none)
    fn join(x: Side, y: Side) -> Side:
        var ends = x.ends + y.ends
        if let u = x.last, let w = y.first, t.is_anc(u, w): ends -= 1
        Side(ends, x.first ?? y.first, y.last ?? x.last)

var st = LazySegTree<t.Pre, Flip<Side>>(
    Flip(Side),
    t.reorder([V] v => if monster[v]: Side(1, v, v) else: Side.empty),
)
print(st.all.now.ends)
repeat read():
    st.apply(t.sub(read<V>()), true)
    print(st.all.now.ends)
