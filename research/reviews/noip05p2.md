# noip05p2

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). Gap compression by `lcm(1..=10)` and the right-to-left DP are the editorial.

## Checklist failures

- Line 31 bounds `fewest` by `0..=100`, restating m's bound where the comment says "at most m stones" (idiomatic-code/R4).
- Line 12 writes `CYCLE = 2520` as a literal; the header derives it as `lcm(1..=10)`.

## Owed rewrites

- Write the dream.
- Type `fewest` as `0..=m`.

## Language gaps

- None.
