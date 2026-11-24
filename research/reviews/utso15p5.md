# utso15p5

Score: 3/5
Dream: 3/5

Cold review: Kruskal is one comprehension over Dsu.unite, each path is a fold over heavy-light chains, and tree roads sit at tree.pos(lower). The in-port generator moved to gen.py with ties and disconnected maps. Top keeps 0 as the identity of max (costs are >= 1) rather than two optionals, which would allocate per fold.

## Cold review (HEAD)

Port 3/5 (no dream.cp, so capped), dream 3/5, from the friction's `## Dream`. The editorial is all here: Kruskal as one comprehension over `joined.unite` (line 21), std `Rooted.chains(u, v, edges: true)` folding a `SegTree` of the two largest distinct costs (lines 39-51), and `best min=` into `?Int` (lines 45-54).
- `both` (lines 25-27) builds a four-element list on every merge to pick the runner-up, and uses `0` as the "no cost" identity. A `(hi: ?Int, lo: ?Int)` with optional max would need neither.
- Line 42 recovers a tree road's lower end with `if tree.parent(u) == Some(v): u else: v`. `Rooted` should hand back an edge's child end (Owner: stdlib, the weighted `Rooted` parent-edge gap).
- `.map(chain => above.fold(chain.range)).fold_from(..., both)` (lines 49-51) is a path fold spelled by hand. It is cheap, but `above.fold_path(tree, u, v)` is what the editorial says.
- Speed: 0.05 s vs 0.04 s at `--max` (1.25x) is over the 1.1x bar. The friction routes it to perf.
