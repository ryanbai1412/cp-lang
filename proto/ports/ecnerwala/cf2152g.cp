// Codeforces 2152 G: Query Jungle.
// After each subtree flip, find the fewest root paths covering every monster.
// A monster needs its own path iff its next monster in preorder is not in its
// subtree, so a lazy segment tree over preorder maintains `(ends, first, last)`
// for both the active monster set and its complement, swapping on subtree flip.

@!multitest
use std.lazy.{Flip, flip_action}
let n: 2..=250_000 = read()
domain V = 1..=n
let monster: [V]Bool = read()
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first)

type Side = (ends: Int, first: ?V, last: ?V)
let empty_side: Side = (0, none, none)

fn join(x: Side, y: Side) -> Side:
    var ends = x.ends + y.ends
    if let u = x.last, let w = y.first, t.is_anc(u, w): ends -= 1
    (ends, x.first ?? y.first, y.last ?? x.last)

let side = Monoid<Side>(empty_side, join)
var st = LazySegTree<t.Pre, Flip<Side>>(
    flip_action<Side>(side),
    t.reorder([V] v => Flip<Side>.of(monster[v], (1, v, v), empty_side)),
)

print(st.all.now.ends)
repeat read():
    st.apply(t.sub(read<V>()), true)
    print(st.all.now.ends)
