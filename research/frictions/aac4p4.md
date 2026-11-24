# aac4p4 friction

## The plan is two generators with the same case split
- **Where:** `port.cp`, `fn must() -> yields Cell` and `fn may() -> yields Cell`. Both start
  with `if trivial: return` and `if let j = last_f:`, then branch on `l <= n` again.
- **What felt wrong:** the dream's single `fn plan() -> ?(must: List<Cell>, may: List<Cell>)`
  states each case once. The port splits it into two generators that repeat the case split
  (idiomatic-code/R4), and repeats its feasibility condition as
  `(trivial or last_f != None or s == f)`.
- **Goal it hurts:** coherence, hard-to-get-wrong (a new case has to be added in three places).
- **Ideal code:** `dream.cp` as written:
  `if let p = plan() and max(l, p.must.len) <= min(r, p.must.len + p.may.len):`
- **Why the port differs:** the dream runs and matches the port's output, but at `--max` it
  materialises the up-to-nm optional cells: dream 0.054 s / 33 MB, port 0.022 s / 3 MB (best
  of 7, same input). The port keeps the cells lazy and only takes `extra` of them.
- **General fix:** a plan whose parts are lazy sequences. A record can already hold a
  generator: `let p = (must: g(), n: 1)`, then `p.must.count()` and `for x in p.must`, runs
  under `cpl run` and builds (09954f019). What is unmeasured is whether a `plan()` that returns
  `(must: yields Cell, may: yields Cell)` stays lazy in the C++. If it does, the port can
  return to one case split at the generators' cost.
- **Severity:** low.
- **Owner:** idioms (this port): try that `plan()` and measure its memory at `--max`.

## Measurements
- Tokens: port 444, original 999.
- `--max` (best of 10): port 0.033 s / 5 MB, accepted C++ 0.078 s / 6 MB. Same output.
- Emitted C++: 18,427 bytes.
