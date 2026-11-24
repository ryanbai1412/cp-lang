# dmopc21c9p4

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A coin-peeling order per row and per column: remove a heads coin, flip its neighbours, and repeat, using the smallest ready coin each time.

## Checklist failures

- `peel` keeps `flipped`, `gone` and `ready` in step by hand inside a `while let u = ready.min()` loop.
- `for nb in [u.prev, u.next]: if let v = nb: if not gone[v]` nests three tests for the live neighbours.
- `peel_slow` repeats the body as an `@alt`, so the two share their bookkeeping by copy.
