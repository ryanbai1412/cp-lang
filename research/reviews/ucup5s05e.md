# ucup5s05e

Score: 4/5
Dream: 4/5

Active port emitted size moved `24,877 -> 26,831 B` (+1,954 B, <48 KB) from updated checked-geometry/slice runtime helpers on current `main`, to be trimmed by dead-helper pruning.

Scored cold against `design/EXEMPLARS-PORTS.md` (previous score 3/5; awaiting rescore).

## Language gaps

Logged in `research/frictions/ucup5s05e.md`:
- `std.seq`: `combinations(k)` for iterating `k`-element subsets of a list.

## Cold review (HEAD)

Dream 3/5, port → 3/5, rescored on current main; sent back.
- Owed: the header asserts that only a few hull vertices around the query's x can matter (lines 3-5), and `near` hard-codes `at + 4`/`hi - 5` (lines 35-36). Derive why five chain vertices on each side, plus one off-diagonal point, suffice; that is the solution's invariant.
- Owed: `quad_edges` (lines 27-28) lists the cyclic hull pairs by hand.
- Reported gap: `combinations(3)` (line 43; the `t[0]` E0305 follows from it).
- Minor: `for _ in 0..q` (line 51) should be `repeat q`.

Port 3/5: a triple loop in place of `combinations` (reported).

## Cold review (HEAD)

Dream 3/5 → 4/5, rescored cold on current main. The header now derives the quadrilateral, why a `WINDOW = 5` chain window gives a strict triangle, and the four-corner fallback; `hull_edges` is cyclic and `WINDOW` is named.
- The only error is `combinations(3)` (line 56; the `t[0]` E0305s follow from it), which is reported.
- Minor: `near` (lines 47-50) clamps a window with `max(0, min(at - 1, hi - WINDOW))`; a clamped `window(around:, len:)` would say it.

Port 3/5: still hard-codes `at + 4`/`hi - 5` (lines 35-36), lists `quad_edges` by hand (lines 27-30), and uses `for _ in 0..q` (line 54), all of which the dream now fixes; the triple loop (lines 43-47) is the reported `combinations` gap.

## Cold review (HEAD)

Port 3 → 4/5, rescored cold on current main; dream 4/5 holds. The port checks with zero errors. The header now derives the local window, and `WINDOW = 5` replaces the `+4`/`-5` magic numbers.
- Minor: the triple loop (lines 59-63) is the reported `combinations(3)`. `hull_edges` (lines 43-46) is built only for the 4-vertex case it's used in; inline it into line 64.
