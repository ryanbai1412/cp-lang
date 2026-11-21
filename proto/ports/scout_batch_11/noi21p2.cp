// NOI 2021 P2, Path Intersection: a layered graph with as many vertices in
// the last layer as in the first. Count the ways to pick vertex-disjoint
// paths from every first-layer vertex to the last layer, each way signed by
// the parity of its crossings. By the Lindstrom-Gessel-Viennot lemma that is
// the determinant of the path-count matrix: the product of the layer matrices.
// original: corpus/submissions/3893923.cpp
use std.mod.{Mod}

const P = 998244353
type Z = Mod<P>
type Mat = List<List<Z>>

fn mul(a: Mat, b: Mat) -> Mat:
    var c = [[Z.zero for _ in 0..b[0].len] for _ in 0..a.len]
    for i in 0..a.len:
        for j in 0..b.len:
            if a[i][j] != Z.zero:
                for k in 0..b[0].len: c[i][k] += a[i][j] * b[j][k]
    c

// Row reduction to an upper triangle; a row swap flips the sign.
fn det(m: Mat) -> Z:
    var (a, d) = (m, Z.one)
    let n = a.len
    for c in 0..n:
        var pivot = c
        while pivot < n and a[pivot][c] == Z.zero: pivot += 1
        if pivot == n: return Z.zero
        if pivot != c:
            swap(a[pivot], a[c])
            d = -d
        d *= a[c][c]
        let inv = a[c][c].inv
        for r in c + 1..n:
            let f = a[r][c] * inv
            for j in c..n: a[r][j] -= f * a[c][j]
    d

let t: 1..=5 = read()
for _ in 0..t:
    let k: 2..=100 = read()
    domain Layer = 0..k
    domain Gap = 0..k - 1
    let size: [Layer]Int = read()  // 1..=200 vertices in each layer
    let edges: [Gap]Int = read()
    var paths: Mat = [[Z(Int(i == j)) for j in 0..size[Layer.first]] for i in 0..size[Layer.first]]
    for g in Gap:
        var step = [
            [Z.zero for _ in 0..size[Layer(g.to_idx + 1)]] for _ in 0..size[Layer(g.to_idx)]
        ]
        for _ in 0..edges[g]:
            let (u, v): (Int, Int) = read()
            step[u - 1][v - 1] = 1
        paths = mul(paths, step)
    print(det(paths))
