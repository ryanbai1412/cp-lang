# yac6p6

Score: 3/5
Dream: 3/5

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

## Cold review (HEAD)

Dream (new) 3/5, port 3/5 (port = dream), cold-reviewed on current main; sent back. It checks with zero errors, and the header derives the anti-diagonal meet-in-the-middle.
- Owed: the skip-or-pay transition is written three times, in `small_dp` (lines 48-56), the 3x3 suffix DP (lines 83-91) and the prefix DP (lines 101-109). Each copy computes `skip` from `k.prev` twice. `step_min` shares only the final `min`. Name one `relax(from: ?…, k)` step, or one DP over a rectangle, and call it three times.
- Owed: the header's own bound, `O((nm / 3) * nm * MAXK)` (line 15), is about `13 333 * 40 000 * 6 ≈ 3e9` cell steps at `n = m = 200`. Either say why only bucketed cells pay and why that fits, or bound it.
- Minor: line 43 rebuilds `Row(x1.ord + i + 1)` from raw offsets. Lines 67, 103 and 106 use comma guards where 4.9 has `and`: `if let … and cond` / `while let … and cond` is the guard spelling (decisions 4.9).
- No friction report is needed: everything owed exists.
