// Source: AtCoder Educational DP Contest R (Walk), https://atcoder.jp/contests/dp/tasks/dp_r
// Problem: a simple directed graph on n <= 50 vertices given as an adjacency matrix.
//   Count the directed walks of length k (k <= 1e18, edges may repeat), modulo 1e9 + 7.
// Solution: the walks of length k from i to j are entry (i, j) of adj^k, so the answer is
//   the sum of all entries of adj^k. `Mat ** k` powers by squaring with n^3 products:
//   O(n^3 log k).
type Z = Mod<1_000_000_007>
let n: 1..=50 = read()
let k: 1..=1e18 = read()
domain V = 1..=n
let adj: [V, V]0..=1 = read()

let walks = Mat<Z>.new([V, V] (i, j) => Z(adj[i, j])) ** k
print(sum(walks.get(i, j) for i in 0..n for j in 0..n))
