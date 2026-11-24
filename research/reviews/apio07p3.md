# apio07p3

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/apio07p3.md`
(there is no `dream.cp`), without the C++. `Window = 0..5` and `View = Subset<Window>` name
the five-pen view, `happy: [Pen, View]Int` is the editorial's table, and the ring DP fixes the
start view and checks `cur[start]` at the end with `[View]?Int`.

## Checklist failures
- idiomatic-code/R4: the fear and love sets are read by two copies of the same loop.
- idiomatic-code/R3: `slide` re-derives `Subset.shifted(-1)` as a loop over five positions.

## Owed rewrites
- `fn read_view(pen, count) -> View`, called twice.
- `slide` on `v.shifted(-1)`.

## Language gaps
- `Subset` is not a `collect` target (E0302, in the friction), so `read_view` keeps a `var`.

## Dream notes
It keeps one `var` loop for the collect gap; 4/5 until `Subset` collects.
