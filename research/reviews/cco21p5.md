# cco21p5

Score: 3/5
Dream: 3/5

The block DP from the header: `earliest`, `found_by`, a suffix-min `later` over cuts, and a lazy `SegTree` of `?Int` minimums with range add. The search for the next block start walks a cut pointer through `Int`, a reported typing gap with repros.

## Checklist failures

- idiomatic-code/R6: `Town.cuts(Int(next) - 1)` twice (line 43) crosses cuts through `Int`; reported with an E0301 repro.

## Owed rewrites

- None until the cut-range search types.

## Language gaps

- `scan` over a reversed table and a search over a range of cuts do not type: research/frictions/cco21p5.md.

## Dream notes

- `dream.cp` keeps the same workaround, `V.cuts((Int(i.r)..=n).first_true(...)!)` (idiomatic-code/R6, caps at 3); the friction's ideal `(i.r..=end).first_true(c => ...)` is the line it should hold.
