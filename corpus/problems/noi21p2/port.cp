// Source: NOI 2021 P2 (Intersecting Paths), https://dmoj.ca/problem/noi21p2
// limits: 2000 ms, 1024 MB
// Problem: T <= 5 tests; a layered DAG with k <= 100 layers, n_1 = n_k <= 100 and
//   n_1 <= n_i <= 2 n_1, edges only from layer j to j + 1. Over all sets of n_1 vertex-disjoint
//   paths from layer 1 to layer k, print (#sets with an even number of crossings) - (#odd),
//   mod 998244353.
// Solution: the crossing parity of a path set is the parity of the permutation it induces
//   from the first layer to the last, so by the Lindstrom-Gessel-Viennot lemma the answer is
//   det of the n_1 x n_1 path-count matrix. Count paths layer by layer, each edge u -> v adding
//   u's counts from every start to v's, O(n_1 sum m_j), then Gaussian elimination mod p,
//   O(n_1^3) (the counts are stored transposed; the determinant is the same).
// original: corpus/submissions/3893923.cpp

const P = 998244353
type Z = Mod<P>
type Mat = List<List<Z>>

// Row reduction to an upper triangle; a row swap flips the sign.
fn det(m: Mat) -> Z:
    var (a, d) = (m, Z.one)
    let n = a.len
    for c in 0..n:
        var pivot = c
        while pivot < n and a[pivot][c] == Z.zero: pivot += 1
        if pivot == n: return Z.zero
        if pivot != c:
            swap(mut a[pivot], mut a[c])
            d = -d
        d *= a[c][c]
        let inv = a[c][c].inv()
        for r in c + 1..n:
            let f = a[r][c] * inv
            for j in c..n: a[r][j] -= f * a[c][j]
    d

@!multitest
let k: 2..=100 = read()
domain Layer = 0..k
domain Gap = 0..k - 1
let size: [Layer]Int = read()  // 1..=200 vertices in each layer
let edges: [Gap]Int = read()
let n1 = size[Layer.first!]
// reach[v][i]: the paths from start i to vertex v of the current layer
var reach: Mat = [[Z(Int(i == v)) for i in 0..n1] for v in 0..n1]
for g in Gap:
    var next: Mat = [[Z.zero for _ in 0..n1] for _ in 0..size[Layer(g.ord + 1)]]
    for _ in 0..edges[g]:
        let (u, v): (Int, Int) = read()
        for i in 0..n1: next[v - 1][i] += reach[u - 1][i]
    reach = next
print(det(reach))
