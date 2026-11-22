# ucup4s24f

Score: 4/5
Dream: 4/5

Baseline review, cold, on main 53a08f6f (port = dream). Both files check with zero errors. `MonoDeque` carries the window minimum, and the header (lines 3-12) derives the residue-covering rule.

## Checklist failures

- Minor: `last_occ: Map<Int, Int>` (line 22) is keyed by a residue in `0..r` with `r <= 500_000`. A `[0..r]?Int` table plus a count says the same thing without hashing.
- Minor: The lazy-deletion `MaxHeap` (lines 23, 35-37) is hand bookkeeping for "max over a set whose entries change". Say in the header why lazy deletion is O(n log n).
- Minor: `a` and `c` (lines 18-19) are unbounded `Int`.

## Rewrites owed with today's language

- None blocking.

## Gaps that need the language

- `MonoDeque` only slides one way, which is reported in `research/frictions/ucup4s24f.md`.
