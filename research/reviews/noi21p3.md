# noi21p3

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). SCC condensation, the out-tree of last predecessors and a per-query virtual tree, as the header derives.

## Checklist failures

- Lines 29-34 rank the topological order into `pos` and pick each parent with `up[a.to] == None or pos[c] > pos[up[a.to]!]`, a hand max-by with an unwrap (idiomatic-code/R3).
- Line 22-23 count component sizes by a loop (`counts`/`buckets` exist).
- Line 46 finds a virtual-tree node by linear `find` per lookup.

## Owed rewrites

- Write the dream.
- Since `order` is topological, `up[a.to] = c` assigned in `order` already keeps the last predecessor; drop `pos`.

## Language gaps

- None.
