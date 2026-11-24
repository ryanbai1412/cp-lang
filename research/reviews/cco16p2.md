# cco16p2

Score: 3/5
Dream: 3/5

Per pivot, the other hares are folded into the upper half plane, sorted by exact `ByAngle` (`arg`), and swept one direction at a time with `runs(by:)`; the header explains why a side takes the pivot plus the points ahead, behind or both. The sweep is the editorial, but it is written as running sums by hand.

## Checklist failures

- idiomatic-code/R3: `seen` is built by a push loop over `Hare` (lines 31-35) where a comprehension says it: `[if upper(v): (v, true, w) else: (-v, false, w) for h in Hare if h != pivot ...]`.
- The friction is stale: it says the port adopted `ByAngle` and `turn` (it imports `arg`), and its snippet cites `proto/ports/cco16p2.cp` with weights `-1e5..=1e5` (the port reads `-10_000..=10_000`).

## Owed rewrites

- Build `seen` with a comprehension and sort it with `sorted(by:)`.
- `var best: ?Int` with `max=` then `best!` is fine; `seen` is never empty at n >= 2.

## Language gaps

- `Pt` cannot carry a coordinate range on `read()`, hence `pos(s)`: research/frictions/cco16p2.md.

## Dream notes

- The dream hides the whole argument (ahead/behind on the line, strict right side) in `sweep_halfplanes`, a one-problem helper; the editorial's sweep should be visible, e.g. a fold over `runs_by_angle` with the `(right, ahead, behind)` step written once (caps at 3).
