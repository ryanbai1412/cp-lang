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
