# atcoder_dp_r: Walk

The answer is the sum of the entries of adj^k.

## Dream

```cp
let walks = Mat<Z>.new([V, V] (i, j) => Z(adj[i, j])) ** k
print(sum(walks.get(i, j) for i in 0..n for j in 0..n))
```

## `Mat ** k` puts std::function in the C++
- **Where:** `port.cp`, `fn times` and the squaring loop over k's bits.
- **What felt wrong:** `Mat.mul` keeps its non-default ring path (std/matrix.cp, `g.add` and
  `g.mul`) even when the ring is the default, so `cpl explain` counts 2 dynamic closures and
  read_only_copies' std_functions cap fails. The port spells the multiply and the power out
  by hand. Sent to Core.
- **Goal it hurts:** costs are visible; the obvious way is the fast way.
- **Ideal code:** the dream above.
- **General fix (core / std):** a default-ring `Mat` drops the ring branch, so it builds no
  std::function.
- **Severity:** medium (the std structure is the one a port has to avoid).
