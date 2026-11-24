# dmopc20c6p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Backward propagation on the reverse graph: a city with no free route is caught, and a caught city frees one route from each predecessor, catching it when it has police or no free route left.

## Checklist failures

- `police: [City]0..=1` is a 0/1 flag compared with `== 1`; a `Bool` read would say it.
- `free`, `caught` and `fresh` are three structures that together are a worklist fixpoint written by hand.
- `print([Int(caught[u]) for u in City])` turns a `Bool` table into digits by hand.
