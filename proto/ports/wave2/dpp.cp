// AtCoder DP P Independent Set: paint each vertex of a tree white or black
// with no two adjacent black vertices. Count the paintings.
type Z = Mod<1_000_000_007>
let n: 1..=1e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first)

// paintings of v's subtree with v white / black
var white: [V]Z = 1
var black: [V]Z = 1
for v in tree.postorder:
    if let p = tree.parent(v):
        white[p] *= white[v] + black[v]
        black[p] *= white[v]
print(white[V.first] + black[V.first])
