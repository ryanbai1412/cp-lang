# atcoder_dp_s: friction report

Port: `corpus/problems/atcoder_dp_s/port.cp`, dream: `corpus/problems/atcoder_dp_s/dream.cp`
(the code is identical).

## Measurements

- Tokens: port 169, accepted C++ 422.
- Emitted C++: 14348 bytes.
- `--max` (|K| = 1e4, d = 100, median of 15): port 55.4 ms / 3.0 MB, accepted 40.1 ms / 19.6 MB (1.38x time).
- Output: identical to the accepted C++.
- perf: 98.6% in `prog::solve()`, all of it the `next[Rem.wrap(r.ord + x)] += below[r]` loop.

## Friction 1: `Rem.wrap` and `Mod` addition pay three reductions per DP step

- Where: `for x in 0..=9: next[Rem.wrap(r.ord + x)] += below[r]`.
- What felt wrong: the source is the editorial, but each step emits
  `cpl::pos(cpl::modn(cpl::iadd(cpl::emod((ll)r + x - ((int)0 + Rem_lo), Rem_len), Rem_len), Rem_len), Rem_len)`:
  a Euclidean mod, a second `modn` of a value already reduced, an overflow-checked add
  and a position check. `Mod_..._add` then does `cpl::iadd(ll(a.v), ll(b.v)) - P`, an
  overflow-checked add of two values below 2^30. The accepted C++ is one `%` and one
  conditional subtract.
- Goal it hurts: R10, "the obvious way is the correct and fast way".
- Ideal code: unchanged.
- General fix: lower `D.wrap(e)` to a single reduction (and to a conditional subtract
  when the range of `e` is under twice `D.count`, as here, `r.ord + x < d + 10`), drop
  the re-check on an already-reduced result, and add `Mod` operands with plain integer
  arithmetic since both are known below `P`.
- Severity: medium (every digit DP and cyclic index pays it).
- Owner: Core (codegen). Repro: `cpl build corpus/problems/atcoder_dp_s/port.cp`, grep `t1 =`.

## Friction 2: K's length is not in `// Bounds:`

- Where: `// Bounds: d: 1..=100` while the read is `Str<1..=10000>`.
- What felt wrong: the statement's `1 <= K < 10^10000` is the length bound, and the
  header states only `d`.
- Goal it hurts: R1.
- Ideal code: `// Bounds: k: 1..=10000 digits; d: 1..=100`.
- General fix: header edit (idioms); the bounds parser could also flag a `Str<a..=b>`
  read missing from Bounds.
- Severity: low.
- Owner: idioms (header), Factory (bounds parser coverage of `Str` reads).
