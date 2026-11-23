# Friction report: dmopc21c9p1

spec.cp keeps x1's feasible set as a range and clips it once per sum:
`ok = ok.clip(if sign < 0: ..c else: 1 - c..)`. The input now carries the
statement's `1 <= a_i <= 1e9` (`read<[n - 1]1..=1e9>()`), and gen.py covers
`a_i = 1` (it drew `a_i >= 2` before).

## An open-ended range with a negative start traps on construction

- **Where:** `ok = ok.clip(if sign < 0: ..c else: 1 - c..)` in spec.cp.
- **What felt wrong:** `1 - c..` is `1 - c..=Int.last` (R04.4.10), "O(1) to
  build", but both `cpl run` and the emitted C++ trap R0202 "range
  `-3..9223372036854775807` span does not fit in an `Int`" when `1 - c < 0`,
  before `.clip` or `.len` is reached. Repro:
  `let c = 4; let r = 1 - c..; print(r.len)` traps on line 2, while
  `(1..).clip(..c)` runs. 4/11 tests and 42/100 generated cases trap; every
  other case agrees with the AC submission.
- **Goal it hurts:** express (the natural code traps), hard-to-get-wrong.
- **Ideal code:** spec.cp as written.
- **General fix:** check the span where a length is asked for (`.len`,
  iteration count), not when a range is built; or store an open end as
  "to the domain's edge" rather than `Int.MAX`. Routed to compiler
  (worklog, "Request from idioms (2026-11-22, corpus `dmopc21c9p1`)").
- **Severity:** blocker (spec.cp traps on valid input).

## Measurements

`cpl build`: 9486 B of C++. The emitted C++ traps on the same 4/11 tests.
