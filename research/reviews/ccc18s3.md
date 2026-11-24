# ccc18s3

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header covers cameras, conveyors and the 0-1 BFS.
- R2: camera rays with `take_while`, then `bfs01` with conveyor moves at cost 0: the editorial.
- R5/R6: `Row`/`Col` domains, `?Dir4` for a belt, `-1` only at print.
- R3: `cells` is a list built to walk the product domain (reported).
- R10: 0.00 s both.

## Owed rewrites

None.

## Language gaps

- Product domains are not iterable and grids have no `position` (in the report).

## Dream notes

Close to the port; the nested `moves` comprehension is dense but readable.
