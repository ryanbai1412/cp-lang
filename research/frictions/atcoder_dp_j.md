# atcoder_dp_j

Ported from `corpus/submissions/4131649.cpp`. Dream: `corpus/problems/atcoder_dp_j/dream.cp`.

## The header lacks its limits line
- **Where:** `port.cp` lines 1-11: there is no `// limits: 2000 ms, 1024 MB`.
- **What felt wrong:** the header is missing the time budget R10 is measured against
  (idiomatic-code/R1), unlike the other atcoder_dp ports.
- **Goal it hurts:** idiomatic-code/R1.
- **Ideal code:** add the line under `Source:`.
- **General fix:** the header edit, with the next `.cp`-edit commit; the lint suggestion is the
  one filed under atcoder_dp_g.
- **Severity:** low.
- **Owner:** idioms (this port).

## No code gaps
The three counts `(a, b, c)` are the DP state, as the editorial has it.
`steps: [Count, Count, Count]F64` is filled with `c`, then `b`, then `a` ascending, and the
header shows that this order computes every neighbour first. `if let a1 = a.prev` replaces the
original's `a > 0` guards.

## Measurements
- Tokens: port 255, original 506.
- `--max` (n = 300, median of 15): port 108 ms / 108 MB, original 394 ms / 436 MB (0.27x).
  The outputs agree to 1e-12 relative: the port prints `3013.6600361782` and the original
  `3013.660036178`, both inside the 1e-9 the statement allows.
- Emitted C++: 15,364 bytes.
