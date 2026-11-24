# dmopc21c7p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Each figure's parity of lit parents is XORed into `fed`; if the lit-reachable part of the graph is acyclic the answer counts figures whose parity disagrees with their state, else -1.

## Checklist failures

- `up: [Figure]0..=1` and `fed: [Figure]0..=1` are 0/1 flags XORed with `^=`; `Bool` tables would say lit and fed.
- `lit_reach` is filled by a loop over `reachable(...)`; a set or a comprehension would make it a `let`.
- `topo(live) == None` tests acyclicity through the absence of an order; `is_dag` would say it.
