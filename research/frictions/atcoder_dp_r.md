# atcoder_dp_r: friction report

Port: `corpus/problems/atcoder_dp_r/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 119, accepted C++ 515.
- Emitted C++: 19489 bytes (19675 before `Mat.sum`).
- `--max` (n = 50, k = 1e18, median of 41, g++ -O2): port 9.9 ms / 3.1 MB, accepted 10.3 ms /
  3.5 MB (0.96x time) since 88fdf8bab; before it the port was 11.5 ms against 9.8 ms (1.17x).
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

## Friction 2: 1.17x at --max on an 11 ms run (fixed)

- Where: `Mat.mul`'s no-ring product loop (std/matrix.cp), reached through `** k`; perf record
  puts 91% of the port's samples in `Mat_S24_25_mul` (88% of the AC's in its `operator*`).
- What felt wrong: the port is over the 1.1x target. The loop already runs under `unchecked:`
  with no index checks; the cost is two reductions per multiply-add (`Mod.mul`'s `% P`, then
  `Mod.add`'s compare), where the AC writes one `(ret + a * b) % mod`.
- Goal it hurts: R10.
- Ideal code: unchanged; the fix is in std or codegen.
- General fix: accumulate a residue dot product lazily. With `P < 2^30` a product is below
  `2^60`, so a u64 row buffer takes 16 products before one `%= P`. Hand-applied to the emitted
  loop it runs 7.3 ms against the AC's 9.8 ms (0.74x, median of 41, same output); fusing the two
  reductions into one `%` (Perf's measurement) reaches parity, 9.8 ms.
- Severity: low (11 ms against a 2000 ms limit).
- Fixed: 88fdf8bab writes a residue add of a product as one `%` (0.96x); 7e3ff18e1 sums a
  residue matrix product unreduced and reduces once per run: 9.1 ms against the AC's 10.6 ms
  (0.87x, median CPU of 15 at --max, same output).
- Owner: Perf (lazy residue product landed in 7e3ff18e1).
