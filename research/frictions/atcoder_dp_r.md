# atcoder_dp_r: friction report

Port: `corpus/problems/atcoder_dp_r/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 119, accepted C++ 515.
- Emitted C++: 19325 bytes (19675 before `Mat.sum`).
- `--max` (n = 50, k = 1e18, median of 21, g++ -O2): port 11.5 ms / 3.1 MB, accepted 9.8 ms /
  3.5 MB (1.17x time). The `Mat.sum` rewrite changes neither time nor output (old port 11.5 ms).
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

## Friction 1: Mat is indexed by Int, not by the port's domain (sum fixed)

- Where: `Mat<Z>.new([V, V] (i, j) => ...)`; until e9b4b0aa5 also the read-back
  `sum(walks.get(i, j) for i in 0..n for j in 0..n)`.
- Fixed: Content added `Mat.sum()` (e9b4b0aa5) and the port prints `walks.sum()`, so it no
  longer reads the power back with raw `0..n` indices.
- Still open: `Mat` does not carry the `V` domain, so `walks[i, j]` cannot take `i, j: V`
  (the dream's `Mat<Z, V>`). Content logged it as a design item waiting on Core's E0309.
- Goal it hurts: R6, only if a port indexes the result.
- Severity: low (this port no longer indexes it).
- Owner: Content (std/matrix.cp), blocked on Core (E0309).

## Friction 2: 1.17x at --max on an 11 ms run

- Where: the whole program; both runs are 10-12 ms, dominated by start-up and 50^3 * 60
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
