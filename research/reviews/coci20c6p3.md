# coci20c6p3

Score: 2/5
Dream: 4/5

Cold review against `dream.cp`. The knapsack over class sizes (lines 22-31) is the header's, with the pair count as a domain and `Pairs.at(...) else: break` stopping a class once it passes k. But the classes are tallied by hand into a `Map` (lines 17-18) on a friction that no longer holds: `Counter` has `values` (`std/hashmap.cp`, `@get pub fn values`), so `words.tally(by: w => w.sorted()).values`, the dream's line 10, removes that bookkeeping.

## Checklist failures

- R3: `var class_size: Map<Str, 0..=2000>` filled by `class_size[w.sorted()] += 1` (lines 17-18), where `tally(by:).values` exists.
- The friction's "a `Counter` has no `values` and cannot be walked" is stale.

## Owed rewrites

- `for size in words.tally(by: w => w.sorted()).values:` in place of lines 17-18 and 25.
- Drop or rewrite the stale friction.

## Language gaps

- None.

## Dream notes

- Reads like the editorial: the header names the grouping and the knapsack, and each class is one table comprehension. `[0..=k]` as a raw range instead of a named domain and `M(if j == 0: 1 else: 0)` for the start are its rough spots.
