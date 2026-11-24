// expect: ok, output "2 3 1\n5"
// The domain bound by `let (domain N, kt)` is the one kt's nodes are keyed by, so it types
// fns over nodes and indexes kt's tables, in both engines.
domain V = 0..3
let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]
let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
let leaves: [N]Int = kt.up(+, leaf: _ => 1)
fn side(v: N) -> Int:
    guard let p = kt.tree.parent(v) else: return 0
    leaves[p] - leaves[v]
let x: N = kt.leaf[0]
print(side(x), side(kt.leaf[0]) + side(kt.leaf[1]), side(kt.leaf[2]))
print(N.count)
