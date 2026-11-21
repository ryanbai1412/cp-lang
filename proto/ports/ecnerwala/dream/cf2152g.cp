// Codeforces 2152 G: after each subtree flip, the fewest root paths that visit every monster.
// A monster needs its own path exactly when its next monster in preorder is not a descendant,
// so a lazy segment tree over preorder keeps (paths, first monster, last monster) and its flip.
@!multitest
domain V = read(1..=2.5e5)
let monster: [V]Bool = read()
let t = Rooted(Graph<V>.undirected(edges: read<[V.count - 1](V, V)>()), V.first)

struct Side(ends: 0..=V.count, first: ?V, last: ?V):
    fn join(x, y):
        var ends = x.ends + y.ends
        if let u = x.last, let w = y.first, t.is_anc(u, w): ends -= 1
        Side(ends, x.first or y.first, y.last or x.last)

var st = LazySegTree<t.Pre, _>(Flip(Side), t.reorder(v => if monster[v]: Side(1, v, v) else: .empty))
print(st.all.now.ends)
repeat read():
    st.apply(t.sub(read<V>()), flip)
    print(st.all.now.ends)
