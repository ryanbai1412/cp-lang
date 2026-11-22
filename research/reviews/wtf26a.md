# wtf26a

Score: pending cold rescore (adopted full Kruskal + overflow-clamp header, single `domain Node = 0..2 * n - 1`, `struct Cut(w, size, parent)`, `[Node]?Cut` eliminating `cut_ng[i] < m` sentinel, and `DsuOf<Node, Int>` toward the 4/5 dream; remaining gap is `KruskalTree`)
Dream: 4/5

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

## Cold review (HEAD)

Dream 4/5 (new), port → 3/5, rescored on current main. The header derives the reconstruction tree, the per-cut costs and the `max_w` clamp.
- `KruskalTree`, `saturating` and `tree.down` (lines 18-23) are reported gaps.
- Minor: `[v => ... for v in tree.non_root]` (line 21) mixes a table builder and a comprehension; neither spelling is accepted with both. `cost.edges(to: tree.parent)` (line 23) is unexplained.

Port 3/5: a hand Kruskal with `Dsu`, a `top` array and `Node(n + e)` ids (lines 17-27); these are the reported gap.
