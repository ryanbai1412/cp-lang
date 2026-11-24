// n: 2048
// let: size = n
use std.rand.{Rng}
use std.tree.{prufer, from_prufer, Centroids, PermTree, PermNode, cartesian_tree}
let n: Int = read()
domain V = 0..n
domain E = 0..n - 1
domain I = 0..n
var rng = Rng(11)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord + 1), v: rpar[V(e.ord + 1)]!)
let wes: [E](u: V, v: V, w: Int) = [E] e => (u: es[e].u, v: es[e].v, w: rng.int(0, 1000000))
var s = 0
let code = prufer(es)
for _ in 0..400:
    s += from_prufer<V>(code).len // timed: tree.from_prufer
var adj: [V]List<V> = []
for e in E:
    adj[es[e].u].push(es[e].v)
    adj[es[e].v].push(es[e].u)
let g = Graph.undirected(es)
let t = Rooted(adj, root: V(0))
var perm: [I]I = [I] i => i
for i in (1..n).rev:
    let j = rng.int(0, i)
    let x = perm[I(i)]
    perm[I(i)] = perm[I(j)]
    perm[I(j)] = x
let arr: [I]Int = [I] i => rng.int(0, 1000000)
let leaf_f: V -> Int = v => 1 // within: tree.KruskalTree.up
for _ in 0..100:
    s += g.spanning_tree().size(V(0)) // timed: tree.spanning_tree
    s += g.dfs_tree(V(0)).tree.size(V(0)) // timed: tree.dfs_tree
    s += if cartesian_tree(arr)[I(0)] == None: 0 else: 1 // timed: tree.cartesian_tree
    s += prufer(es).len // timed: tree.prufer
    s += Centroids(adj).root.ord // timed: tree.Centroids.new
    s += LcaTable(t).n // timed: tree.LcaTable.new
    s += PermTree(perm).first.len // timed: tree.PermTree.new
let (domain N, kt) = kruskal_tree(wes, w: e => wes[e].w)
for _ in 0..40:
    let (domain M, k2) = kruskal_tree(wes, w: e => wes[e].w) // timed: tree.kruskal_tree
    s += k2.leaf[V(0)].ord
    s += kt.up(+, leaf: leaf_f)[kt.leaf[V(0)]] // timed: tree.KruskalTree.up
    s += kt.tree.size(kt.leaf[V(0)]) // timed: tree.KruskalTree.tree
let cd = Centroids(adj)
for _ in 0..100:
    for x in cd.piece(cd.root): s += x.dist // timed: tree.Centroids.piece
let lt = LcaTable(t)
let pt = PermTree(perm)
let top_f: E -> Bool = e => wes[e].w < 500000 // within: tree.KruskalTree.top
for i in 0..64 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    s += (cd.parent(a) ?? a).ord // timed: tree.Centroids.parent
    s += cd.depth(a) // timed: tree.Centroids.depth
    for x in cd.ancestors(a): s += x.dist // timed: tree.Centroids.ancestors
    s += lt.lca(a, b).ord // timed: tree.LcaTable.lca
    s += lt.dist(a, b) // timed: tree.LcaTable.dist
    s += kt.reach(a, 500000).ord // timed: tree.KruskalTree.reach
    s += kt.top(a, top_f).ord // timed: tree.KruskalTree.top
    s += kt.bottleneck(a, b) ?? 0 // timed: tree.KruskalTree.bottleneck
    let lf = pt.leaf(I((i * 31) % n)) // timed: tree.PermTree.leaf
    s += pt.span(lf).count() // timed: tree.PermTree.span
    s += pt.vals(lf).count() // timed: tree.PermTree.vals
    let x = pt.root
    s += pt.left(x).ord // timed: tree.PermTree.left
    s += pt.right(x).ord // timed: tree.PermTree.right
    s += if pt.node(lf) == PermNode.Leaf(I(0)): 1 else: 0 // timed: tree.PermTree.node
    s %= 1000000007
print(s)
