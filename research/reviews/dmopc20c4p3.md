# dmopc20c4p3

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

A sweep from the top: roombas by descending y, the cords sorted tallest last and popped with `pop_while`, standing cords counted per x in a `Fenwick` over the compressed xs, and each roomba adds the count at or left of its x.

## Checklist failures

- `xs.index_of(c.x)` looks up each cord's compressed index again; `compress` could hand back the index per cord.
- `if let reach = xs.last_le(r.x): touching += standing.prefix(reach.r)` mixes a position and a cut (`reach.r`) to say "at or left of x".
