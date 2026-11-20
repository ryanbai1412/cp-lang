// AtCoder DP V Subtree: for every vertex v of a tree, count the connected
// sets of black vertices that contain v, modulo m (not necessarily prime).
let (n, m): (1..=1e5, 2..=1e9) = read()
type Z = Mod<m>
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first)

// down[v]: such sets inside v's subtree
var down: [V]Z = 1
for v in tree.postorder:
    if let p = tree.parent(v): down[p] *= down[v] + 1

// up[v]: 1 for leaving everything outside v's subtree white, plus the sets
// out there that contain v's parent. m has no inverses, so `excl_fold` gives
// each child the product over its siblings without dividing.
var up: [V]Z = 1
for v in tree.preorder:
    for (c, rest) in tree.excl_fold(v, *, c => down[c] + 1): up[c] = up[v] * rest + 1
for v in V: print(down[v] * up[v])
