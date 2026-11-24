# shoi16p2

Score: 3/5
Dream: pending (no dream.cp; the friction's `## Dream` is the target)

Cold review on main 377955a60, against the idiomatic checklist. Rereviewed after this commit's change to lines 37-39: the four rotations are now a data list summed once. Checks clean, agrees with the reference at max in 0.67x its time and 0.05x its memory. The header (lines 3-10) gives the inclusion-exclusion and the 2 / c share per meeting. No dream.cp, so the port is capped at 3.

## Checklist failures

- The pair loop (lines 53-59) mixes ordered pairs (`continue` on `i == j`) and unordered ones (`i < j`); in the friction report (stdlib, `pairs(ordered:)`).

## Rewrites owed with today's language

- None.

## Gaps that need the language

- `pairs(ordered: true)`.
