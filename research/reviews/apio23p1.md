# apio23p1

Score: 2/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp` and the friction's `## Dream`, without the C++. The
header derives the multi-source start (every reachable reset country), the layer per halving and
the 70-layer cap from the 1e-6 tolerance. The first two searches use std `dijkstra`. The halving
layers, though, run a second Dijkstra written by hand.

## Checklist failures
- idiomatic-code/R3: lines 48-56 hand-roll Dijkstra (`var gains = MinHeap<(F64, C)>()`,
  `while let (d, u) = gains.pop(): if d > best[u]!: continue`), next to `times` on line 31,
  which already calls std `dijkstra`. The dream reruns `dijkstra` seeded by the current distances
  plus the halved ones. The friction gives no measurement or repro for why the port differs.
- idiomatic-code/R4: each relaxation's candidate is written twice: `t / 2 + r.w` (lines 43-44)
  and `d + r.w` (lines 54-55).

## Owed rewrites
- Seed `times` with the finite `best` entries and the halved candidates, as the dream does. If
  that is too slow, measure it and report an incremental `dijkstra(from:, dist:)` to stdlib.
- Bind each candidate once before comparing and storing it.

## Language gaps
- `Graph.only(...)` not implementing `WeightedNext`, which forces the `out` copy (lines 26-29):
  in the friction, with a repro.
- `opt.map(...) ?? []` element typing inside `flat_map`: in the friction.
- No local stub for `@grader fn`: in the friction.

## Dream notes
- `dream.cp` loops `min(k, 80)` while its header says "~70" and the port derives 70: one bound
  in two forms (caps at 3).
- `dream.cp` expands each road into both directions by hand (`let both = [...]`) where the
  friction's dream names `Graph.undirected(...).only(...)`. The friction's version is the
  target.
