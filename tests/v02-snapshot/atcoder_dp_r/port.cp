// Source: AtCoder Educational DP Contest R (Walk), https://atcoder.jp/contests/dp/tasks/dp_r
// Problem: a simple directed graph on n <= 50 vertices given as an adjacency matrix.
//   Count the directed walks of length k (k <= 1e18, edges may repeat), modulo 1e9 + 7.
// Solution: the walks of length k from i to j are entry (i, j) of adj^k, so the answer is
//   the sum of all entries of adj^k. Binary powering with an n^3 matrix product:
//   O(n^3 log k).
// Written out today: the matrix product and the binary power. Builds; stress OK against
//   4131862.cpp.
const P = 1000000007
type Z = Mod<P>
let n: 1..=50 = read()
let k: 1..=1e18 = read()
domain V = 1..=n
let adj: [V, V]0..=1 = read()
fn mul(a: [V, V]Z, b: [V, V]Z) -> [V, V]Z:
    [V, V] (i, j) => sum(a[i, m] * b[m, j] for m in V)
var step: [V, V]Z = [V, V] (i, j) => Z(adj[i, j])
var walks: [V, V]Z = [V, V] (i, j) => Z(Int(i == j))
var e = k
while e > 0:
    if e % 2 == 1: walks = mul(walks, step)
    step = mul(step, step)
    e /= 2
print(sum(walks[i, j] for i in V for j in V))
