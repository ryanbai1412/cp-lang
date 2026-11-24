# ccc21s4

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the walk BFS from station N and the min SegTree over stops.
- R2: BFS over reversed walkways, then the best ride-then-walk stop kept under swaps: the editorial.
- R3: the reversed adjacency is filled by hand (L29-30) for want of `group(pairs, over: D)`; linked from the report now (filed in ccc20s2).
- R3: the `SegTree` is built from a function it then keeps (reported).
- R5: `?Int` for "cannot walk", printed at the end.

## Owed rewrites

None.

## Language gaps

- A `SegTree` built from a function keeps the function (in the report).
- No `group(pairs, over: D)` adjacency builder (linked to ccc20s2's report).

## Dream notes

`bfs(group(... over: Station), ...)` is the right ask.
