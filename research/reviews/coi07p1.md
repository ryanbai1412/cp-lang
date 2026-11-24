# coi07p1

Score: 3/5
Dream: none

The port reads like the header's editorial: one greedy sweep maintains the least feasible time and the furthest reachable endpoint. Lines 22–24 directly express the half-gap correction with `let more = (x - t - p - k) / 2`, `p = x - t - more`, and `t += more`. The code is compact, but the missing dream caps the score at 3; the reported runtime is also six times the accepted baseline.

## Checklist failures
- idiomatic-code/R10: 0.18 s against the C++'s 0.03 s on `gen.py --max`; now filed in the friction (within the 600 ms limit).

## Owed rewrites
- Supply the missing dream; no algorithmic rewrite is owed.

## Language gaps
- Ranged real types (in the friction, now with a fresh `cpl run`: `let k: 0.0..=1e9 = read()` is E0302 "expected `Int`, found `F64`"): L13–14 read K and the positions as plain `F64`.
- The 6x runtime on the sweep (L18) is filed in the friction; the cause is not isolated.

## Dream notes
None: there is no `dream.cp` and no `## Dream` in the friction report, so the port is capped at 3.
