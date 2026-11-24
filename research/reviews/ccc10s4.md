# ccc10s4

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: Source, Problem, Solution and Bounds; the Solution derives "the cheaper of the two spanning trees".
- R2: the dual graph and std `kruskal`, as the editorial; the open-side map pairs each side once.
- R5: `min(spanning(...), spanning(...))!` relies on `None` being `min`'s identity on `?Int` (`min(None, Some(3))` prints 3 under `cpl run`), so a disconnected pens-only forest falls back correctly.
- R6: `outside` is region 0 of `Region` (reported as the missing domain sum).
- R3: `take.count(t => t) == regions - 1` (L31) re-derives "the forest spans" beside `kruskal`; now in the friction report.
- R10: 0.00 s both.

## Owed rewrites

None.

## Language gaps

- Domain sum `Pen + {outside}` for the sentinel region (in the report).
- `zip` is free, not a method (in the report).
- `kruskal` cannot say whether it spans (added, with a `cpl run`: E0310).

## Dream notes

`Pen.flat_map(pen => let m = read<3..=8>(); ...)` reads input inside a lazy pipeline (R7): read the pens first, then group the sides. Otherwise the grouped sides and `spanning: true` read like the editorial.
