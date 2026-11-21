# ucup5s05e

Score: 3/5

The header (lines 1-5) states why only a few hull vertices matter. The hull is hand-built with an array and a `len` counter, the triple search is three nested index loops, and spelling is mixed. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Hand monotone chain (lines 16-23) with a preallocated array and `len`; `std.geometry.lower_hull` exists but drops the ids.
- Hand window widening (lines 40-42) for "up to five around `at`".
- Triple search as three index loops with an early `return` (lines 49-55).
- `side: Int` as `±1` (line 16) instead of an enum or `by:` option.
- Mixed spelling: free `sorted(..., by:)`, `first(...)`, `all(...)` (lines 13, 28, 56) next to `.rev` and `.to_list()`.
- `(i + 1) % 4` cyclic index (line 57); the formatter split of `turn(...)` (lines 29-33) is unreadable.

## Owed rewrites

- Method spellings; one `sides` list for the hull edges instead of `(i + 1) % 4`; clamp the window with one `max`/`min`.

## Language gaps

No friction report exists. Missing and owed as a friction: `lower_hull`/`convex_hull` over keyed points (a `by:` option that keeps the payload), and `k`-subsets of a list (`combinations`).
