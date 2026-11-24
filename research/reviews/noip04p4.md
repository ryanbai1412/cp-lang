# noip04p4

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). Column-by-column backtracking with the forced sum letter and the `plausible` prune follow the header.

## Checklist failures

- Line 31 is one long `not (if ... else: false)` inside `all(...)`; `any(...)` over the columns that are fully known and contradict reads as the header says (idiomatic-code/R8).
- Lines 16-17 and 39 index the three words by raw `0..3` and `0..2`, and line 17 reverses columns as `Col(n - 1 - c.ord)` (idiomatic-code/R6).
- Lines 22-25, 43-45 and 51-53 undo assignments by hand: reported (friction 1).

## Owed rewrites

- Write the dream.
- Name the word roles (an enum or a domain `Word` with `Sum` last) and read the columns units-first once.

## Language gaps

- None.
