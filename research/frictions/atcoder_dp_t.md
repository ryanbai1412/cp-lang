# atcoder_dp_t: friction report

Port: `corpus/problems/atcoder_dp_t/port.cp`, dream: `corpus/problems/atcoder_dp_t/dream.cp`
(identical code).

## Measurements

- Tokens: port 162, accepted C++ 475.
- Emitted C++: 14066 bytes.
- `--max` (n = 3000, median of 15): port 7.8 ms / 3.0 MB, accepted 9.8 ms / 2.5 MB (0.79x time).
- Output: identical to the accepted C++.

## Friction 1: the modulus is spelled without separators

- Where: `const P = 1000000007` then `type Z = Mod<P>`.
- What felt wrong: every other DP port writes `Mod<1_000_000_007>`; this one names the
  constant once and never uses `P` again, so the name adds a line and the digits are
  ungrouped.
- Goal it hurts: R8 (one spelling), R13.
- Ideal code: `type Z = Mod<1_000_000_007>`.
- General fix: port edit; the lint could warn on a numeric literal of 7+ digits with no
  `_` separators.
- Severity: low.
- Owner: idioms (port), Consumers (literal-grouping lint).

Otherwise the prefix/suffix-sum step reads as the editorial: `..=placed` and
`(..placed).rev` are the two directions and `run` the running sum.
