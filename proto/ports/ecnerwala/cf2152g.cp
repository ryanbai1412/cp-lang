// A monster needs its own path exactly when no other monster lies in its subtree, so the
// answer counts the monsters whose next monster in preorder is outside their subtree.
// A lazy segment tree over preorder keeps that count with the first and last monster, for
// the current monster set and for its complement; flipping a subtree swaps the two.
@!multitest
use std.lazy.{Flip, flip_action}
let n: Int = read()
domain V = 1..=n
let monster: [V]Bool = read()
let edges: [0..n - 1](V, V) = read()
let t = Rooted(Graph.undirected(edges), V.first)

type Side = (ends: Int, first: ?V, last: ?V)

fn join(x: Side, y: Side) -> Side:
    var ends = x.ends + y.ends
    if let u = x.last, let w = y.first, t.is_anc(u, w): ends -= 1
    (ends, x.first ?? y.first, y.last ?? x.last)

let side = Monoid<Side>((0, none, none), join)
var st = LazySegTree<t.Pre, Flip<Side>>(
    flip_action<Side>(side),
    t.reorder([V] v => Flip<Side>.of(monster[v], (1, v, v), side.id)),
)

print(st.all.now.ends)
repeat read():
    st.apply(t.sub(read<V>()), true)
    print(st.all.now.ends)
