// A monster needs its own path exactly when no other monster lies in its subtree, so the
// answer counts the monsters whose next monster in preorder is outside their subtree.
// A lazy segment tree over preorder keeps that count with the first and last monster, for
// the current monster set and for its complement; flipping a subtree swaps the two.
@!multitest
let n: Int = read()
domain V = 1..=n
let monster: [V]Bool = read()
let edges: [0..n - 1](V, V) = read()
let t = Rooted(Graph.undirected(edges), V.first)

type Side = (ends: Int, first: ?V, last: ?V)
type Node = (now: Side, flipped: Side)

fn join(x: Side, y: Side) -> Side:
    var ends = x.ends + y.ends
    if let u = x.last, let w = y.first, t.is_anc(u, w): ends -= 1
    (ends, x.first ?? y.first, y.last ?? x.last)

let none_side: Side = (0, none, none)
let flip = Action<Node, Bool>(
    id: false,
    compose: (f, g) => f != g,
    apply: (f, x, span) => if f: (x.flipped, x.now) else: x,
    over: Monoid<Node>(
        (none_side, none_side),
        (x, y) => (join(x.now, y.now), join(x.flipped, y.flipped)),
    ),
)
fn leaf(v: V) -> Node:
    let one: Side = (1, v, v)
    if monster[v]: (one, none_side) else: (none_side, one)
var st = LazySegTree<t.Pre, Node>(flip, t.reorder([V] v => leaf(v)))

print(st.all.now.ends)
repeat read():
    let v: V = read()
    st.apply(t.sub(v), true)
    print(st.all.now.ends)
