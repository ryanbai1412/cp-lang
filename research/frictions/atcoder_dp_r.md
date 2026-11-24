# atcoder_dp_r: friction report

Port: `corpus/problems/atcoder_dp_r/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 119, accepted C++ 515.
- Emitted C++: 19675 bytes.
- `--max` (n = 50, k = 1e18, median of 15): port 13.3 ms / 3.1 MB, accepted 11.1 ms / 2.8 MB (1.20x time).
- Output: identical to the accepted C++.

## Dream

```cp
type Z = Mod<1_000_000_007>
let n: 1..=50 = read()
let k: 1..=1e18 = read()
domain V = 1..=n
let adj: [V, V]0..=1 = read()

let walks = Mat<Z, V>.new((i, j) => Z(adj[i, j])) ** k
print(walks.sum())
```

## Friction 1: Mat is indexed by Int, not by the port's domain

- Where: `print(sum(walks.get(i, j) for i in 0..n for j in 0..n))`.
- What felt wrong: the adjacency is `[V, V]`, but the power is read back with raw `0..n`
  indices, so the port names the vertex space twice and leaves the 0-based / 1-based
  shift to the reader. `Mat` has `get`, `at`, `product` and `transpose` but no way to sum
  or iterate its entries.
- Goal it hurts: R6 (one domain per index space), R8 (the walk count is "sum of all
  entries", one spelling).
- Ideal code: `walks.sum()`, or `Mat` indexed by `V` so `walks[i, j]` takes `i, j: V`.
- General fix: let `Mat` carry its index domain (or expose its entries as a `[D, D]T`
  view), and add `sum` alongside `product`.
- Severity: low.
- Owner: Content (std/matrix.cp).

## Friction 2: 1.20x at --max on a 13 ms run

- Where: the whole program; both runs are 11-13 ms, dominated by start-up and 50^3 * 60
  multiplications.
- What felt wrong: the port is over the 1.1x target. After 6525aa9fc the power emits no
  `std::function`; the remaining difference is the `Mat` product loop checking each
  `get`/`set` index, which the accepted C++'s raw `long long[50][50]` does not.
- Goal it hurts: R10.
- Ideal code: unchanged; the fix is in codegen.
- General fix: drop the bounds checks in `Mat.mul`'s inner loop, whose indices are
  provably in `0..n`.
- Severity: low (13 ms against a 2000 ms limit).
- Owner: Core (codegen), repro `cpl build corpus/problems/atcoder_dp_r/port.cp` and the
  `--max` input.
