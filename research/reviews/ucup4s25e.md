# ucup4s25e

Score: 4/5
Dream: 4/5

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 3/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup4s25e.md`:
- `std.geometry`: `best_dot` on a monotone upper hull without CW->CCW reversal.

## Cold review (HEAD)

Dream 4/5, port → 4/5, rescored on current main. The header derives the Minkowski view of cross-boundary subarrays.
- The only error is `best_dot` (line 13), which is reported.
- Minor: the two child merges (lines 27-30) are one loop written twice. `pref` is a hand prefix loop (lines 19-20) where a `scan` over the points says it. `solve_dc` takes raw `Int` bounds.

Port 4/5: the dream, with `hull_extreme` over a reversed hull for the reported `best_dot`.
