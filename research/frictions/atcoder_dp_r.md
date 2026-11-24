# atcoder_dp_r — Walk

The walks of length k from i to j are entry (i, j) of adj^k; sum every entry of `Mat ** k`.

## Dream

```cp
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

let walks = Mat(adj.map(Z)) ** k
print(walks.sum())
```

## Mat forgets the domains of the table it was built from
- **Where:** `port.cp`, lines 13-14:
  ```cp
  let walks = Mat<Z>.new([V, V] (i, j) => Z(adj[i, j])) ** k
  print(sum(walks.get(i, j) for i in 0..n for j in 0..n))
  ```
- **What felt wrong:** the matrix is built from a `[V, V]` table but is read back with 0-based
  `Int` pairs, so the vertex domain is lost and `0..n` reappears. Repro:
  `let walks = Mat<Z>.new([V, V] (i, j) => Z(1)) ** 3; print(walks[V.first!, V.first!])`
  gives `error[E0302]: expected \`Int\`, found \`V\``. There is no `sum` over all entries and
  no elementwise `map` to lift `[V, V]0..=1` into `Z`.
- **Goal it hurts:** distinct index spaces as domains; express.
- **Ideal code:** `let walks = Mat(adj.map(Z)) ** k` then `print(walks.sum())`, with
  `walks[i, j]` for `i, j: V`.
- **General fix (stdlib, matrix):** `Mat<R, C, T>` keyed by the domains of its table
  (`[R, C]T` in, `[R, C]T` indexing out), plus `sum`/`map` over entries.
- **Severity:** medium.
