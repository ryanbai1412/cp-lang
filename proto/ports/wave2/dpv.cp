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
// out there that contain v's parent. m has no inverses, so each child gets
// the product over its earlier siblings, then over its later ones.
var up: [V]Z = 1
for v in tree.preorder:
    var earlier = up[v]
    for c in tree.children(v):
        up[c] = earlier
        earlier *= down[c] + 1
    var later: Z = 1
    for c in tree.children(v).rev:
        up[c] = up[c] * later + 1
        later *= down[c] + 1
for v in V: print(down[v] * up[v])
