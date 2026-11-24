# dmopc21c2p4

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A DP over runs of equal height with three states per run (alone, spilling into the next run, pulled by the next run), each the fewest pours.

## Checklist failures

- `alone`, `spills` and `pulled` are three parallel `?Int` states plus `last`, updated in one loop; a small struct or enum of the three cases would name the DP.
- `let h = run[run.keys.first!]` reads a run's height through its first key.
- `ways` is a `List<(done, cells)>` rebuilt per run only to take three minima over it.
- `pours(cells)` returns 0 for `cells <= 0`, so negative cell counts (`run.len - k - 1`) flow through as a sentinel.
