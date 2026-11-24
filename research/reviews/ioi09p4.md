# ioi09p4

Score: 3/5
Dream: 4/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-9) gives the block DP, and lines 19-30 fill it smaller blocks first, with 2D prefix sums for the block's raisins; the dream is the same recurrence `@memo`ised.
- `min(by_row ?? by_col!, by_col ?? by_row!)` (lines 27-29) combines two optional minima by hand, but `min(?a, ?b)` already skips `None`. `min(by_row, by_col)!` gives the same output, and the friction now marks the entry fixed with a fresh run. This is bookkeeping the language removes, so the score is 3 until the rewrite.
