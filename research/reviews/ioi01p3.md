# ioi01p3

Score: 3/5
Dream: 4/5

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-10) derives the staircase invariant, and rank/unrank (lines 42-72) skip whole blocks of completions as it says.
- `weight` hard-codes the powers of 6 (line 24), next to `% 6` (line 28), `0..7776` (line 21) and `r * 5` (line 33): the side 5 and radix 6 are each written several times (idiomatic-code/R4). The base-6 packing is in the friction.
- The memo is a hand-kept `[Key]?Int` cleared per count (lines 23, 39); in the friction (`@memo` has no reset).
- `let rank: Int = read()` (line 58) has no bound; the statement gives 1..2^31.

## Dream notes

- The dream is the port with `@memo(reset:)` and the staircase `[Row]0..=5` as the key; that removes the packing and the hand memo, and needs `domain Row` for the side once.
