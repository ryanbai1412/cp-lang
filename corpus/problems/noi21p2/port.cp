// Source: NOI 2021 P2 (Intersecting Paths), https://dmoj.ca/problem/noi21p2
// limits: 2000 ms, 1024 MB
// Problem: T <= 5 tests; a layered DAG with k <= 100 layers, n_1 = n_k <= 100 and
//   n_1 <= n_i <= 2 n_1, edges only from layer j to j + 1. Over all sets of n_1 vertex-disjoint
//   paths from layer 1 to layer k, print (#sets with an even number of crossings) - (#odd),
//   mod 998244353.
// Solution: the crossing parity of a path set is the parity of the permutation it induces
//   from the first layer to the last, so by the Lindstrom-Gessel-Viennot lemma the answer is
//   det of the n_1 x n_1 path-count matrix. Count paths layer by layer in a std `Mat<Z>`
//   (a row per vertex, a column per start), each edge u -> v adding u's row to v's,
//   O(n_1 sum m_j), then std `det` of the transposed counts, O(n_1^3).
// Bounds: k: 2..=100; size: 1..=200; edges: 0..=40000; u: 1..=from; v: 1..=to
// original: corpus/submissions/3893923.cpp

type Z = Mod<998_244_353>

@multitest
let k: 2..=100 = read()
domain Layer = 0..k
domain Gap = 0..k - 1
let size: [Layer]1..=200 = read()
let edges: [Gap]0..=40_000 = read()
let first = size[Layer.first!]
var paths = Mat<Z>.identity(first)
for g in Gap:
    let (from, to) = (size[Layer(g.ord)], size[Layer(g.ord + 1)])
    var next = Mat<Z>.zeros(to, first)
    repeat edges[g]:
        let (u, v): (1..=from, 1..=to) = read()
        for s in 0..first: next.at(v - 1, s) += paths.get(u - 1, s)
    paths = next
print(paths.det())
