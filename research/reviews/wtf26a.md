# wtf26a

Score: 2/5

The header (lines 1-3) states the plan. The Kruskal reconstruction tree is hand-built from three parallel `cut_*` arrays over raw `Int` node ids with `n + e` offsets, and the overflow clamp (line 40) is not derived. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Hand reconstruction tree (lines 14-32): `label`, `n + e` ids and parallel `cut_w`/`cut_sz`/`cut_ng`.
- Raw `[0..m]` node space (lines 14-17) that line 49 re-declares as `domain V = 0..(2 * n - 1)`; one domain, declared once.
- `cut_ng[i] < m` as a "has parent" sentinel (line 34).
- Unexplained clamp (lines 12, 40): `max_w` and `max_w / cut_sz[i]`.
- Mixed spelling: free `sorted(...)` (line 37) next to `.sorted(...)` (line 11); `Int(bool)` seed (line 51).
- Unbounded `n` and `w` (lines 8, 10).

## Owed rewrites

- One `Node` domain with a `?Node` parent; one `Cut` record per node; method `sorted`; header deriving the clamp; a dream.

## Language gaps

No friction report exists. Missing and owed as a friction: a Kruskal reconstruction tree (DSU merge tree) in the stdlib.
