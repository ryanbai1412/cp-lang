# cco18p6

Score: 3/5
Dream: 3/5

The header's merge sort by run reversals is there: `reverse`, `merge` and `sort_range`, then a's flops and b's in reverse. It is the C++ shape: global `cur`/`flops` reset per call and raw `Int` positions throughout.

## Checklist failures

- idiomatic-code/R7: `cur` and `flops` are shared globals reset inside `sorting_flops` (lines 16-17, 50-51); reported.
- idiomatic-code/R6: positions and range ends are raw `Int` (`type Flop = (l: Int, r: Int)`, `merge(l: Int, mid: Int, r: Int)`), not a position domain; not in the friction.

## Owed rewrites

- A `domain Pos = 0..n` for `cur` and the flops, so the 1-based print is `print(f.l, f.r)` once over `there + back.rev`.

## Language gaps

- The sort state is global; a lent parameter cannot become it (E0603): research/frictions/cco18p6.md.

## Dream notes

- `merge_sort(merge: ...)` and `rotate_by_reversals(mid, record: flops)` are invented one-problem APIs, and `record:` hides the push to `flops` inside a call (idiomatic-code/R7, caps at 3). No header.
