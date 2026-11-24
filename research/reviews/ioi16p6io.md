# ioi16p6io review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and `corpus/problems/ioi16p6io/dream.cp`.

Same program as ioi16p6: outermost spans, a `LiChao` of photo lines with `(val, count)` keys so
ties go to fewer photos, and `aliens` over the fee. The `MonotoneHull` workaround is reported
with its repro.

## Checklist failures

- The header says "For fixed lam a DP from the right" (line 9); the port runs left to right
  over `spans.with_prev`, with `best` the cost of the spans so far (R1).
- `let (lo, x) = (Int(s.l), Int(s.r))` (line 27) where decision §101 names `.start`/`.end`
  (R8); the dream does the same.

## Language gaps

- `MonotoneHull` tie-breaking: in research/frictions/ioi16p6io.md.

## Dream notes

- At bar (identical to ioi16p6's dream).
