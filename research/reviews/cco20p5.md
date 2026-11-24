# cco20p5

Score: 2/5
Dream: 4/5

Online: `SortedBag`s give the best overlapping pair and the shortest interval, and a `SegTree` with a `Gaps` monoid gives the best disjoint pair, all with `?Int` for missing values. Close to the editorial, but `least` hand-rolls a `min` that exists.

## Checklist failures

- idiomatic-code/R13: `fn least` (line 21) is `min` on `?Int`: under 0.2 `min(Some(2), None)` is `Some(2)` and `min(Some(5), Some(2))` is `Some(2)`; `None` only wins when both are `None`. The friction's "no `min` over `?Int` that skips `None`" does not hold.
- idiomatic-code/R5: `Gaps` fields and the bag keys are `Int` though every coordinate is an `X`; only spans need `Int`.

## Owed rewrites

- Replace `least` with `min` (three calls) and drop the friction bullet.

## Language gaps

- The emitted C++ is about 54 KB (W0201): research/frictions/cco20p5.md.

## Dream notes

- `dream.cp` is the old offline timeline; the friction's online fragment is the target and reads like the editorial.
