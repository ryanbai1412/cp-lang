# ucup4s25e

Score: 3/5

Re-reviewed cold on main 2cca08a5 after the port changed. Port: 3/5, capped (no dream). The hand two-pointer is gone: each query is `hull_extreme` (line 29), and the `best` seed is explained (line 12). Left: `solve_dc` still writes the global `best` (line 22) instead of returning its maxima; `hull.reverse()` (line 26) has no stated reason (which end `hull_extreme` expects); `Pt(v, 1)` is built twice (line 29); the negated prefixes `-pref[i]` over `(lo..md).rev` (line 20) aren't derived; `n` and `q` are unbounded (line 7). No friction report exists. Earlier notes below are superseded where they conflict.

The header (lines 1-3) names the plan, and the body uses `upper_hull` and `minkowski_upper` from `std.geometry`. But `try_max` writes `maxs` as a side effect (line 14), `maxs` is seeded with prefixes as a stand-in identity (line 12), and the offline queries walk the hull with a hand two-pointer (lines 29-36). No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Hidden mutation (line 14, used at 23): `try_max` updates a global from inside the divide step.
- Stand-in initial value (line 12): `maxs[i] = pref[i + 1]` without saying it is the best subarray of that length starting at 0.
- Hand query sort and pointer walk (lines 29-36) instead of a hull query or `bsearch_first` per query.
- `Pt(i + 1, ...)` x is a length and y a prefix sum (line 11); neither is named.
- Unbounded inputs (lines 7-8).

## Owed rewrites

- Return the per-length maxima from `solve_dc` and merge with `max`; answer each query by binary search on hull slopes; a dream.

## Language gaps

No friction report exists. Possibly owed: a `best_dot` query on an upper hull in `std.geometry`.
