# noi19p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). One `LiChao(min, ...)` per station and a time sweep, as the header says.

## Checklist failures

- Lines 17-24 add a dummy train m (1 -> 1 at time 0) so the answer sits in `after[Train(m)]`; a fake input row standing for "start" (idiomatic-code/R5).
- Line 29 sorts latest-first by negating times (`-trains[i].s`) instead of `desc`.
- Line 30 destructures the sort key `(_, arrival, i)`: the event kind is a `Bool` named by position.

## Owed rewrites

- Write the dream.
- Sort events with `desc(time)` and an enum kind (`Depart | Arrive`) whose order puts departures first.

## Language gaps

- None.
