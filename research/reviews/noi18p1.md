# noi18p1

Score: 4/5
Dream: 4/5

Cold review on main c8a46b201: port, dream and friction report, not the C++. The port is the dream minus three reported spellings: Dijkstra home, a Kruskal reconstruction tree by descending altitude, and `kt.up(min, ...)` for the least walk under each node.

## Checklist failures

- Lines 28-30 read `v0`, `p0` one at a time inside `for _ in 0..q` where the dream reads `[q](V, 0..=s)` rows; `repeat q` is the counted-loop spelling elsewhere in the corpus.

## Owed rewrites

- None.

## Language gaps

- Weight and key by field projection: friction 1.
- `desc(p + 1)` for "altitude > p": friction 2.
- `.ring.step` for the forced-online decode: friction 3.

## Dream notes

- The dream in the friction report has no header; it leans on the port's.
