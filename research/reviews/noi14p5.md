# noi14p5

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (`dream.cp`). The shuffle, the swaps, then values taken in increasing order while their column fits the row's allowed range (lines 45-54): the header's greedy, with cells kept in 32 bits to fit 256 MB (line 25).

## Checklist failures

- The inverse permutation is a hand loop (lines 39-40): reported (`inverse()` of a product-keyed array is rejected).
- idiomatic-code/R6: cells are flat `Int` indices split with `/ m` and `% m` (lines 49-50), and `lo`/`hi` are `[0..n]Int` (lines 45-46); the dream has `R`, `Col` and `Cell.nth`.

## Owed rewrites

- Row and column domains for `lo`, `hi` and the split.

## Language gaps

- In the friction.

## Dream notes

`value.inverse()` and `Cell.nth(s.ord)` state the layout; the dream's `rand()` stands for the statement's generator.
