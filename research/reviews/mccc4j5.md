# mccc4j5

Score: 4/5
Dream: 4/5

Cold review: Kahn's order driven by a min-heap of finish times, with the header's argument for why the last pop is the answer (lines 23-34).

## Checklist failures

- R4: `time`, `users` and `left` (lines 14-21) are parallel tables filled from one read loop; `time` starts at a filler `1`.

## Owed rewrites

- Read `[V](time: 1..=1e4, deps: List<V>)` in one typed read (works today) and derive `users` and `left` from it.

## Language gaps

- A `dag_fold` over the part Kahn reaches when the graph has cycles: in research/frictions/mccc4j5.md.

## Dream notes

- None.
