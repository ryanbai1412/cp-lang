# atcoder_dp_w

Score: 4/5
Dream: 4/5

Cold review against the idiomatic checklist.

- R1: no `limits:` line.
- R2: bucket rules by right end, `best.set(at, best.fold(..at))`, then `best.apply(l..=at, a)` is the editorial.
- R3: std `SegTree` with `max` and `add`, no hand lazy tree (the accepted C++ is 1919 tokens of one).
- R5: every read bounded, including `a: -1e9..=1e9`.
- R6: `Pos` and `Last` are distinct spaces; the label conversions are explicit (friction 1).
- R10: 0.62x.

## Checklist failures

R1 (limits).

## Owed rewrites

Header `limits:` line.

## Language gaps

"Domain plus one point" has no spelling (friction 1, low).
