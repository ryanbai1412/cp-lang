# 996warmup8p3 friction

The port is `dream.cp` (same code), so there is no dream/port gap. One gap is shared by both.

## A per-item binding inside a comprehension
- **Where:** `port.cp` and `dream.cp`, `movers`:
  `[if to_right: s.at else: L - s.at for b in Ball for s in [place(b, t1 % (2 * L))] if s.right == to_right]`
- **What felt wrong:** `for s in [place(...)]` builds a one-element list just to name a value
  that the filter and the element both use.
- **Goal it hurts:** express.
- **Ideal code:** `[if to_right: s.at else: L - s.at for b in Ball let s = place(b, t1 % (2 * L)) if s.right == to_right]`
- **Run:** that line under `cpl run` (09954f019):
  `error[E0103]: expected ']' to close the comprehension, found 'let'`.
- **General fix:** a `let` clause in comprehensions and generator expressions (core syntax).
  It would also cover the `for x in [f(y)]` idiom in other ports.
- **Severity:** low.
- **Owner:** Core (syntax).

## Measurements

- Judge limits: unknown, so the port has no `limits:` line. https://dmoj.ca/api/v2/problem/996warmup8p3 answers
  404 (not public), a DMOJ problem search finds no 996 Warmup problem, and meta.json and the repo
  hold no statement. Tools use the 2000 ms / 256 MB default until someone with the statement sets it.
- Tokens: port 546, original 903.
- `--max` (best of 10): port 0.016 s / 6 MB, accepted C++ 0.060 s / 5 MB. Same output.
- Emitted C++: 19,323 bytes.
