# atcoder_dp_v: friction report

Port: `corpus/problems/atcoder_dp_v/port.cp`, dream: `corpus/problems/atcoder_dp_v/dream.cp`
(identical code).

## Measurements

- Tokens: port 126, accepted C++ 673.
- Emitted C++: 21356 bytes.
- `--max` (n = 1e5, median of 15): port 28.9 ms / 15.8 MB, accepted 30.5 ms / 14.0 MB (0.95x time).
- Output: identical to the accepted C++.

## Friction 1: the runtime modulus is spelled as `% m` in two closures

- Where: `Monoid(1, (a, b) => a * b % m)` and `(x, _, _) => (x + 1) % m`.
- What felt wrong: M is read, so `Mod<P>` cannot be used, and the port reduces by hand
  in both the monoid and the lift. std has `Barrett` (std/mod.cp:353) for exactly this
  case (`1 <= m < 2^31`, here `m <= 1e9`), but it is a multiplier, not a value type, so
  it cannot be the `T` of a `Monoid` the way `Mod<P>` can. The values are still plain
  `Int`, so nothing stops an unreduced value from reaching `print`.
- Goal it hurts: R5 (the residue is not a type), R8 (two spellings of "mod m").
- Ideal code:
  ```cp
  type Z = ModN(m)          // residue type for a modulus read at run time
  let ways = tree.reroot(Monoid(Z(1), (*)), (x, _, _) => x + 1, (u, _) => u)
  ```
- General fix: a runtime-modulus residue type in std/mod.cp (backed by `Barrett`) with
  the same operators as `Mod<P>`.
- Severity: medium (every "mod M, M not prime and read" problem hand-reduces today).
- Owner: Content (std/mod.cp).

The header has neither `limits:` nor `original:` (header edit, idioms).
