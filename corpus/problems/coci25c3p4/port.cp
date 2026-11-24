// Source: COCI 2025/26 Round 3 (Drzava), https://dmoj.ca/problem/coci25c3p4
// limits: 2000 ms, 512 MB
// Problem: a tree on n <= 3000 cities whose diameter is below 36 roads. A state is a capital
//   and a set of other secondary cities, none of which lies on the path from the capital to
//   another. For every size k = 1..n print the number of states with k cities, mod 1e9 + 7.
// Solution: per capital the counts form a polynomial (x^j: j secondary cities), and disjoint
//   parts multiply. A part hanging off an edge either takes its top city alone or picks inside
//   its children's parts: down[v] = hung(product of the children's down). Rerooting: around[v]
//   multiplies the children's down with hung(around[parent] / down[v]), dividing out a factor
//   that starts with 1. Sum around[v] over v. Rooted at vertex 1 the depth is below 36, so the
//   products cost O(36 n^2) in all.
// original: corpus/submissions/7730380.cpp
// Bounds: n: 1..=3000
const P = 1000000007
type Z = Mod<P>
let n: 1..=3000 = read()
domain V = 1..=n
domain E = 1..n

// Parents among the first 17 labels keep every city within 17 roads of city 1.
fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, min(e.label, 17))), V(e.label + 1))

let edges: [E](u: V, v: V) = read(gen: random_tree)
let tree = Rooted(Graph.undirected(edges), V.first!)
// Fewer than 36 roads between any two cities, so none is 36 roads below the root.
let height: 0..=35 = V.map(v => tree.depth(v)).max()!

// A count for every k is a polynomial: the coefficient of x^k is the count.
// Picking independently in two parts multiplies their polynomials (`convolve`).

// Take the part b back out of a product a. b starts with 1 (pick nothing).
fn without(a: List<Z>, b: List<Z>) -> List<Z>:
    var q = [a[i] for i in 0..a.len - b.len + 1]
    for i in 0..q.len:
        for j in 1..=min(i, b.len - 1): q[i] -= b[j] * q[i - j]
    q

// A part hanging off an edge: pick inside it, or pick its top vertex alone.
fn hung(inside: List<Z>) -> List<Z>:
    var f = inside
    if f.len == 1: f.push(Z(0))
    f[1] += 1
    f

// down[v]: picks in v's subtree, hanging from v's parent
var down: [V]List<Z> = [V] _ => [Z(1)]
for v in tree.at.rev:
    var inside = [Z(1)]
    for c in tree.children(v): inside = convolve(inside, down[c])
    down[v] = hung(inside)

// around[v]: picks with the tree hung from v, one factor for every
// neighbour. The parent's factor is everything around the parent except v.
var around: [V]List<Z> = [V] _ => [Z(1)]
var total: [n]Z = 0
for v in tree.at:
    for c in tree.children(v): around[v] = convolve(around[v], down[c])
    if let p = tree.parent(v): around[v] = convolve(around[v], hung(without(around[p], down[v])))
    for k in 0..around[v].len: total[k] += around[v][k]
print(total)
