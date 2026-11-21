# ucup4s25e

Score: pending (was 3/5: derived `-pref[i]` over `(lo..md).rev` and Minkowski `(length, sum)` in header, made `solve_dc` return its per-length maxima eliminating global `best` mutation and fake seed, documented CW->CCW `hull.reverse()`, extracted `answer(v)` avoiding duplicate `Pt(v, 1)`, bounded `(n, q)`, added dream + friction for `best_dot`)
Dream: pending (added dream using `best_dot(hull, Pt(v, 1))` directly on `upper_hull`)

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 3/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup4s25e.md`:
- `std.geometry`: `best_dot` on a monotone upper hull without CW->CCW reversal.
