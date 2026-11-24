# dmopc21c6p4

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

Two BFS runs from a to b, one avoiding d and one avoiding c; the shorter decides which colour the other roads get.

## Checklist failures

- The `match (skip_d, skip_c)` over four cases picks the minimum of two optionals with a flag; `min` over the present ones, keyed by length, would say it.
- `touches(r, x)` is a helper for `x in (r.u, r.v)`.
- `let rest = if found.met_d: 2 else: 1` encodes the choice as a colour number.
