# ioi17p3 review

Score: 3/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi17p3.md).

The outer loop is the editorial's: Arezou's attractor of the charging stations, Borzou's
attractor of the rest, drop it, repeat. The attractor itself is a hand BFS over reverse edges
with live out-degree counters, a standard game algorithm std lacks; that and `[V]Bool` sets
are reported.

## Checklist failures

- `attract` (lines 25-42) hand-rolls the attractor (R3); reported.
- The header says "Marks give O(1) clears" (line 11), but each `attract` call allocates fresh
  `exits`, `won` and `queued` tables and recounts live out-degrees from `tracks`; there are no
  marks (R1). The bound O(n (n + m)) still holds.

## Language gaps

- `Subgraph.attractor` and set algebra over `V`: in research/frictions/ioi17p3.md.

## Dream notes

- At bar: `on.attractor(target, owner, player)`, `alive -= bad + attract(...)`.
