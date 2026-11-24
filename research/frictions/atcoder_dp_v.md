# atcoder_dp_v: friction report

Port: `corpus/problems/atcoder_dp_v/port.cp`, dream: `corpus/problems/atcoder_dp_v/dream.cp`
(identical code).

## Measurements

- Tokens: port 126, accepted C++ 673.
- Emitted C++: 21356 bytes.
- `--max` (n = 1e5, median of 15): port 28.9 ms / 15.8 MB, accepted 30.5 ms / 14.0 MB (0.95x time).
- Output: identical to the accepted C++.

## Friction 1: the runtime-modulus type was not found from the docs

- Where: the previous port wrote `Monoid(1, (a, b) => a * b % m)` and `(x, _, _) => (x + 1) % m`.
- What felt wrong: M is read, and the port reduced by hand in both closures because the
  porter (and this report's first draft) looked for a residue type in std/mod.cp and found
  only `Barrett`, a multiplier. The type exists in the language: `type Z = Mod(m)` for a
  read `m` (decision 232, spec/conformance/core-03/runtime-modulus-read.cp), with `+ - *`
  and folds. Nothing in std/mod.cp's docs or the porting handbook points to it.
- Goal it hurts: writable from the docs; R5 and R8 in every port that hand-reduces.
- Ideal code (now the port):
  ```cp
  type Z = Mod(m)
  let ways = tree.reroot((*), lift: (x: Z, _, _) => x + 1, finish: (u, _) => u)
  ```
- General fix: a pointer from `Mod<P>` and `Barrett` in std/mod.cp to `Mod(m)`, and a
  line in the idioms cookbook for "modulus read at run time".
- Severity: low (docs), fixed in this port.
- Owner: Content (std/mod.cp docs).

The rewrite agrees with the accepted C++ (`cpl test` 6/6) and measures 29.8 ms / 15.8 MB
against 31.3 ms / 14.0 MB (0.95x). The header has no `limits:` line
(header edit, idioms).
