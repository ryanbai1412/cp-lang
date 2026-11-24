# dmopc21c6p3

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

A multi-source BFS for k rounds: painted cells, in paint order, spread their colour to unpainted 4-neighbours.

## Checklist failures

- `front` and `next` are a hand-rolled BFS frontier swapped each round; a layered `bfs` from the painted cells capped at depth k would state it.
- `var paint` is read and then painted in place, so the input and the answer share one name.
