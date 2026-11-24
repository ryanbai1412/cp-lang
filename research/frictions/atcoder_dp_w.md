# atcoder_dp_w: friction report

Port: `corpus/problems/atcoder_dp_w/port.cp`, dream: `corpus/problems/atcoder_dp_w/dream.cp`
(identical code).

## Measurements

- Tokens: port 224, accepted C++ 1919.
- Emitted C++: 32616 bytes.
- `--max` (n = m = 2e5, median of 15): port 103.8 ms / 21.5 MB, accepted 168.4 ms / 33.5 MB (0.62x time).
- Output: identical to the accepted C++.

## Friction 1: two domains for one position space, converted by label

- Where: `domain Last = 0..=n`, `Last(i.label)` and `Last(l.label)`.
- What felt wrong: `Last` is `Pos` plus a "no 1 yet" state 0. The port converts three
  times by label. That is correct, and a `?Pos` would not work as a SegTree index, but
  the relation "Last is Pos with a state before it" is written as arithmetic on labels
  rather than stated once.
- Goal it hurts: R4 (the offset fact is restated at each conversion).
- Ideal code: `domain Last = Pos.with_before` (hypothetical) giving `Last.of(i)`, or keep
  the explicit labels.
- General fix: none needed now; this is the second port (after atcoder_dp_l's
  `I.ranges`) where a domain is "another domain plus one point", worth a design note if a
  third appears.
- Severity: low.
- Owner: Core (language design note), no code change requested.

The header has no `limits:` line (header edit, idioms).

## Addendum: open ranges, and the emitted size
- **Open ranges (fixed in this port and its dream):** `best.fold(Last.first!..at)` and
  `best.fold(Last.first!..=Last.last!)` are now `best.fold(..at)` and `best.fold(..)`, which
  `SegTree` already accepts.
- **The emitted C++ is over the judge's size target.** `cpl build` warns W0201: with comments
  the C++ is 34,583 bytes, over `generic`'s 32,768-byte target. It fits (31,968 bytes) only
  because the build drops the comments. A five-line repro (`domain P = 0..=n`;
  `SegTree<P, Int, Int>(max, fill: 0, act: add)`; one `apply(.., 1)`; `print(t.fold(..))`)
  emits 29,207 bytes, against 6,549 for `print(n)` alone. solve() calls only `new`, `apply` and
  `fold`, yet 25 SegTree functions (12,245 bytes) are emitted. They include the growing-tree
  paths (`grow`, `open_node`, `blank_at`, `kid`, `dense_width`), which a dense fixed domain never
  uses. Goal it hurts: submission. Severity: medium. Owner: Core (codegen); repro sent.
  **Fixed by f74d4e0dd and 443583f46** (codegen prunes fold branches on list fields that nothing
  reachable fills). On 0bed2e3b3 the port emits 28,405 bytes for `--judge atcoder` and 28,183
  for `generic`, with no W0201. Perf measured the dense repro at 29,929 → 23,989 bytes.
- **Measurements after the rewrite:** port 196 tokens; `--max` best of 3: port 0.13 s / 22.2 MB,
  accepted 0.47 s / 34.5 MB, same output. `corpus.py verify`: 0 findings.
