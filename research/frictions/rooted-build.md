# Rooted construction (`bench/trees/root_bench`)

Not a port: the stdlib's own `Rooted` build, measured against hand C++ (CSR,
BFS order, heavy-first preorder) in `bench/trees/root_bench.{cp,cpp}` with the
edges read over a declared edge domain (`domain E = 0..n - 1`). Time is at
1.1x already; peak memory is 2x, and both causes below are core.

## Ranged array elements are stored as 64-bit
- **Where:** `stdlib/tree.cp`, `Rooted`: `dep: [V]Int`, `sz: [V]Int`,
  `k0: [V]Int`, `k1: [V]Int` hold values in `0..=V.K`.
- **What felt wrong:** there is no way to say so that changes storage.
  Even a top-level `var d: [V]0..=1_000_000` emits `std::vector<ll>`; width
  selection (decisions.md table row 1.2) only narrows scalar struct fields
  (`narrow_ints` in `cpl-codegen/src/scope.rs`). In a generic struct the bound
  would also have to come from the domain parameter (`[V]0..=V.K`), whose
  size is a runtime value with a static cap from the domain's declaration.
- **Goal it hurts:** performance ("values take the fewest bytes their types
  allow").
- **Ideal code:** `dep: [V]0..V.K`, `sz: [V]1..=V.K`, stored as `int32_t`
  because every vertex domain is input-sized (its positions are already
  `int`).
- **General fix:** core: array element types pick their width from their
  range like scalar fields do, and a range bounded by a domain's `K` takes
  the domain's position width. Helps every `[V]Int` count, depth and size
  table in the stdlib and the ports.
- **Severity:** medium.

## Array arguments are copied into the struct a function returns
- **Where:** `stdlib/tree.cp`, `Rooted.finish(roots, par, dep, far, bfs, k0, k1)`
  ends with `Rooted<V, P>(root, par, dep, far, sz, bfs, k0, k1, ...)`.
- **What felt wrong:** the emitted C++ takes the arrays as `const&` and copies
  `par`, `dep`, `far`, `bfs`, `k0` and `k1` into the struct, although the
  caller never uses them again. Locals of `finish` (`sz`, `heavy`, `top`,
  `tin`, `at`) are moved. The helper cannot be inlined into its two callers
  because it declares the preorder domain `P`.
- **Goal it hurts:** performance (abstractions cost what direct C++ costs).
- **Ideal code:** unchanged.
- **General fix:** codegen: pass an array argument by value and `std::move`
  it at the caller's last use, so a helper that stores its arguments costs
  nothing. Same class as the centroid copies in `bench/trees/README.md`.
- **Severity:** medium.

## Measurements
n = 10^6, seed 1, best of three, identical output. Hand edits are to the
emitted C++ only.

| version | time | RSS |
|---|---:|---:|
| hand C++ | 0.24 s | 70 MB |
| `cpl build` | 0.28 s | 138 MB |
| + `dep`, `far`, `sz`, `k0`, `k1` as `int` | 0.24 s | 99 MB |
| + arguments moved into `finish` | 0.24 s | 75 MB |

Emitted C++: 15759 bytes.
