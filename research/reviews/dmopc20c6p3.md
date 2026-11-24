# dmopc20c6p3

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

Backward propagation on the reverse graph: a city with no free route is caught, and a caught city frees one route from each predecessor, catching it when it has police or no free route left.

## Checklist failures

- `police: [City]0..=1` is a 0/1 flag compared with `== 1`; a `Bool` read would say it.
- `free`, `caught` and `fresh` are three structures that together are a worklist fixpoint written by hand.
- `print([Int(caught[u]) for u in City])` turns a `Bool` table into digits by hand.
