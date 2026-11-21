# yac6p6

Score: 2/5

Baseline review, cold, on main 96359635. No editorial header; lines 1-2 give only provenance. The DP is written three times (the small case, the forward 3x3 and the backward prefix), with the transitions spelled out in each. The `(x2 + y2) % 3` diagonal bucketing (line 40), the step at which every query is split, is never explained. No dream exists, so the score is capped at 3.

## Checklist failures

- No header: why queries split on the diagonal `(x2 + y2) % 3` and why `MAXK = 5` (lines 1-4, 40).
- The same min-plus transition is written three times (lines 25-32, 58-64, 74-80); a single `step` helper or one DP over a direction removes it.
- `if let nr = r + 1 if nr <= x2` (lines 27, 30, 75, 78) is domain stepping done by hand; `r.next` bounded by the query rectangle says it.
- `continue` after a base case (lines 24, 39, 73) instead of an expression-valued case.
- `id: Int` in the query record (line 11) where `0..q` is the domain.
- Inline `FRICTION 39.4` comments (lines 17, 26) with no friction report.

## Rewrites owed with today's language

- Write the header, factor the transition once, and type the query id.

## Gaps that need the language

- `?Int + Int` in min-plus DP (the author's FRICTION 39.4): tropical `?Int` arithmetic exists elsewhere in the corpus (ioi13p3), so check whether the guards are still needed. No friction report exists; flag it.
