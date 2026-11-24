# nccc10s4

Score: 3/5
Dream: 3/5

Cold review: correct, and the header proves the exact-point and spent-corner arguments, but most of the port is a hand line arrangement (`Line`, `Hom`, its `Ord`, corner ids through a `Map`) and the walk is a global-state backtracker. All of it is reported; the port is 1.9x the C++ (4.9 s against 2.6 s), with the numbers in the friction.

## Checklist failures

- R7: `walk` updates the globals `used`, `spent`, `sides` and `polygons` (lines 54-80), so the count only reads right with the undo loops beside it.
- R4: the first-side loop (lines 83-90) repeats the `spent` set-then-walk of `walk`'s inner loop.

## Owed rewrites

- Return the count from `walk(corner, start, sides) -> Int` instead of the global `polygons` and `sides`, as the dream does.

## Language gaps

- No exact line arrangement, walking both ways along a line with a self-undoing `spent`, and `domain Corner` built from a map's size: all in research/frictions/nccc10s4.md.

## Dream notes

- The dream's last line is `...`, so the first-side loop that fixes each polygon's count once is not written, and `run.take_while_unspent(spent)` marks corners spent inside an expression (a hidden side effect). It should write the first-side loop and make the spending a `with` scope.
