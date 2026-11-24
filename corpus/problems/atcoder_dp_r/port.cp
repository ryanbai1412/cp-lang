// Source: AtCoder Educational DP Contest R (Walk), https://atcoder.jp/contests/dp/tasks/dp_r
// Problem: a simple directed graph on n <= 50 vertices given as an adjacency matrix.
//   Count the directed walks of length k (k <= 1e18, edges may repeat), modulo 1e9 + 7.
// Solution: the walks of length k from i to j are entry (i, j) of adj^k, so the answer is
//   the sum of all entries of adj^k, by squaring over k's bits with n^3 products:
//   O(n^3 log k).
type Z = Mod<1_000_000_007>
let n: 1..=50 = read()
let k: 1..=1e18 = read()
domain V = 1..=n
let adj: [V, V]0..=1 = read()

type Walks = [V, V]Z
fn times(a: Walks, b: Walks) -> Walks = [V, V] (i, j) => sum(a[i, l] * b[l, j] for l in V)

// adj^k by squaring: `walks` holds adj^(the bits of k seen so far)
var walks: Walks = [V, V] (i, j) => Z(if i == j: 1 else: 0)
var step: Walks = [V, V] (i, j) => Z(adj[i, j])
for bit in 0..60:  // k < 2^60
    if k.bit(bit): walks = times(walks, step)
    step = times(step, step)
print(sum(walks[i, j] for i in V for j in V))
