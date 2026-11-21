@!multitest
domain V = read(1..=2.5e5)
let monster: [V]Bool = read()
let t = Rooted(Graph<V>.undirected(edges: read<[V.count - 1](V, V)>()), V.first)

// a monster needs its own path exactly when its next monster in preorder is not a descendant
record Side(ends: Sum<0..=V.count>, first: First<V>, last: Last<V>):
    fn join(x, y) = Side(x.ends + y.ends - t.is_anc(x.last?, y.first?), x.first or y.first, y.last or x.last)

var st = LazySegTree<t.Pre, _>(Flip(Side), t.reorder(v => Side.of(v) if monster[v] else Side.id))
print(st.all.now.ends)
repeat read():
    st.apply(t.sub(read<V>()), flip)
    print(st.all.now.ends)
