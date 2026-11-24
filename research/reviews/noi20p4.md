# noi20p4

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). Greedy over a `SortedBag`, and the m = n - 2 split as a bitset subset sum traced back, both derived in the header.

## Checklist failures

- Line 13 spells the case marker `@multitest` (idiomatic-code/R8).
- Lines 26 and 31 print `j.ord + 1`: the 1-based ingredient number is rebuilt from a 0-based domain; `domain Ing = 1..=n` prints itself (idiomatic-code/R6).
- Line 42 shifts subset sums by `-mn` into `Off` by hand; the offset is derived on lines 35-36.

## Owed rewrites

- Write the dream.
- Use `domain Ing = 1..=n` and `@!multitest`.

## Language gaps

- None.
