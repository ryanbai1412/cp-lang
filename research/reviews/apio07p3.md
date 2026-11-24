# apio07p3

Score: 4/5
Dream: 4/5

Cold review of `port.cp` (the dream under `## Dream` in `research/frictions/apio07p3.md` is
now the port), without the C++. `Window = 0..5` and `View = Subset<Window>` name the
five-pen view, `happy: [Pen, View]Int` is the editorial's table, each child's sets are one
`View.of(...)` over the read pens, `slide` is `shifted(-1)` plus the new pen, and the ring DP
fixes the start view and checks `cur[start]` at the end with `[View]?Int`.

## Checklist failures
None. Short of 5/5 only because `(x - pen) % n` maps a pen to its window position by
arithmetic; a cyclic pen domain would say it, which is a design item, not a port fault.

## Owed rewrites
None.

## Language gaps
None open (`Subset.of`, 80bde58b9, closed the collect gap).
