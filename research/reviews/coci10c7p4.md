# coci10c7p4

Score: 4/5
Dream: 4/5

Cold review against the friction's `## Dream`. The header's two-pointer argument (lines 6-9, restated at 34-35) is the loop: the floor walks the sorted heights, the ceiling only rises and is found by `first_true` (lines 36-45), and `delivers` asks whether a fill from P inside `[lo, hi]` reaches every K (lines 21-32).

## Checklist failures

- R3: the fill keeps its own stack and `seen` grid (lines 24-31); reported.
- R1, minor: line 4 says heights `< 1e6`, while the Bounds line and the read say `1..=1e6` (lines 11, 16).

## Owed rewrites

- Make line 4 agree with the Bounds line.
- The friction's "repro sent to Core" quotes no run; add the `cpl run` of the dream's `reachable(...)` line (R11).

## Language gaps

- `reachable` over `(Row, Col)` cells (no domain for a grid cell): in the friction.
- Every cell of a grid without a comprehension (line 17): in the friction.

## Dream notes

- `delivers` as `reachable` plus an all-houses check, judged as the port with it; reads like the editorial.
