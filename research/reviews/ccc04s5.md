# ccc04s5

Score: 4/5
Dream: 4/5

Cold review against the idiomatic checklist, of the port re-ported from `dream.cp`.

- R1: complete; the Solution states the per-column up/down walk and the `None` start.
- R2: column DP left to right, one straight run per column: the editorial.
- R4: one scan, `for step in [-1, 1]`, instead of separate up and down loops.
- R5: `here: [R]?Int`, `None` for unreachable rows; no sentinel.
- R6: rows are `R`, columns `Col`; the walk position `q` is an `Int` converted with `R(q)`
  after the `q in 0..rows` guard, so the conversion is visibly in-domain (R12).
- R10: 0.92x.
- Short of 5/5: `q += step` walks an `Int`, where a domain step (`R.at(q)` in a `while let`)
  would keep the walk in `R`.

## Checklist failures

None.

## Owed rewrites

None.

## Language gaps

None. (`a[1] max= 5` on a `[0..3]?Int` slot that holds `None` stores 5 under `cpl run`, so
port and dream use `leave[R(q)] max= got`.)
