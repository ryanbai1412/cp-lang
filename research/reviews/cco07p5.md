# cco07p5

Score: 3/5
Dream: 3/5

The strategy reads as the header: a seeded std `Rng` picks blocks until one `Look` hits, then the candidates `(p, v)` are halved by a `Look 0 mid` at their median and advanced by comprehension until one is left.

## Checklist failures

- idiomatic-code/R4: the block's end `min(n, lo + block)` is written on lines 26 and 29.

## Owed rewrites

- Name the block once (`let hit_block = lo..min(n, lo + block)`), and pass it to `look` and to the candidate comprehension.
- `cands.sort()` each round only to read the median; a selection call would say it, but `xs.select(k)` is not a value-returning std call under 0.2 (it type-checks as a statement only), so the sort stays until std has one.

## Language gaps

- `ask` reply needs a `Str` binding before the compare: research/frictions/cco07p5.md.
- Ring positions are `Int` with `% n`: same report.
- No value-returning k-th smallest, so the median is a full sort per round: research/frictions/cco07p5.md, "No selection of the k-th smallest" (added by this review).

## Dream notes

- `rng.int(0, (n - 1) / block) * block` and `min(n, lo + block)` are each written twice (idiomatic-code/R4, caps at 3); the port's `random_block()` already removes the first.
- `block` and `rng` are used but never defined, and `cands.select(...)` is not in the friction as a gap.
- A dream under 4 is sent back and caps the port at 3 (research/reviews/README.md); the port itself reads at 4.
