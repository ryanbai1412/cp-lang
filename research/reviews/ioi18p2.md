# ioi18p2 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi18p2.md).

Reads like the editorial: corners plus dents per prefix, a std `SegTree` of `MinCount` with
range add, and a swap re-accounting the two seats and their neighbours. The hand-made range-add
action and the hand-written neighbour offsets are reported.

## Checklist failures

- `arrival(y - 1, x)` ... and `[(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)]` (lines 50, 62) spell
  the grid neighbours twice with `Int` coordinates (R6); reported.

## Language gaps

- A std range-add action on `MinCount` and grid neighbours: in research/frictions/ioi18p2.md.

## Dream notes

- The port with three lines replaced (`act: add`, `who.get(cell)`, `grid.around`); at bar.
