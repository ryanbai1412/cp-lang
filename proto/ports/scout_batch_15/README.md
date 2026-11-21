# Scout batch 15: flow, matching, Eulerian tours, and graph connectivity

Ten hard, unported submissions from `corpus/submissions/` spanning max-flow,
min-cut, weighted bipartite matching / min-cost flow, lexicographically
smallest matching, bipartite 2-coloring, cyclic Eulerian tours, vertex-
biconnected components / bridges / articulation points, SCC condensation +
virtual trees, and rollback bipartite connectivity. None overlaps
`proto/ports/` or any other scout batch (`scout_batch_1`–`scout_batch_16` or
`blind_critic_1`). The friction backlog is in
[`research/friction-backlog-batch-15.md`](../../../research/friction-backlog-batch-15.md)
(items `N1`–`N11`).

Every port runs today on the reference interpreter (`proto/cp.py lib-run`) and
is differentially checked against its compiled original C++ submission on
40 random small inputs each (400 runs total), plus unit tests for the two
proposed library modules:

```sh
python3 proto/cp.py test proto/ports/scout_batch_15      # 5 library tests
python3 proto/ports/scout_batch_15/check.py              # all 10 ports, 40 random inputs each
python3 proto/ports/scout_batch_15/check.py cco16p3 100  # one port, 100 inputs
python3 proto/cp.py lib-run proto/ports/scout_batch_15/dmopc20c7p5.cp < input
```

Inputs in `check.py` are small so the Python interpreter finishes quickly;
checks verify exact answers (and witness validity for multi-answer problem
`coci25c3p3`, plus an exact brute-force oracle for `noi09p1` because the
original `5102817.cpp` infinite-loops on unsolvable self-loop inputs).
`orig_4798070` (`noi16p2`) allocates ~50 MB structs on the stack, so `check.py`
raises `RLIMIT_STACK` before running C++ originals.

## Proposed library modules (`lib.toml`, imported as `scout.*`)

These two files are proposed userland additions for Stage-2 review rather than
edits to `stdlib/`:

- `connectivity.cp`: iterative `two_edge_components` (handling parallel edges
  via `a.e` and returning a fresh domain `C` so `std.graph.condense` builds
  the bridge tree directly), `articulation_points`, `biconnected_components`
  (vertex-biconnected blocks), `virtual_tree` on `std.tree.Rooted<V>`,
  and `two_color` (rollback parity union-find is now `std.dsu.ParityDsu`).
- `flow_ext.cp`: `MinCostFlow` (SPFA + Johnson potentials + Dijkstra) and
  `max_weight_matching` (O(n^3) Hungarian), cross-tested against each other on
  random signed matrices.

## Ports and LOC comparison

Totals across the 10 ports: **700** non-blank, non-comment CP Lang lines vs
**2,029** raw C++ lines (**65.5%** reduction) and **1,261** library-stripped
C++ lines excluding templates, preambles, `main()`, and standard data
structures (**44.5%** reduction).

- `noi09p4.cp` (48 vs 156 raw / 59 stripped; `5144326.cpp`): max-flow /
  min-cut + cycle pruning. Replaces the original's O(V(V + E)) repeated DFS
  with O(V + E) reverse-dependency Kahn peeling, then max-weight closure via
  `std.flow.FlowGraph`.
- `dmopc20c7p5.cp` (20 vs 105 raw / 80 stripped; `4370353.cpp`): project
  selection min-cut. Replaces a 5000 x 5000 dense matrix and manual capacity
  scaling with sparse Dinic via `std.flow.FlowGraph`.
- `sdoi17r1p4.cp` (27 vs 168 raw / 28 stripped; `5466798.cpp`): fractional
  programming + weighted bipartite matching. Replaces 90 lines of `__int128`
  dense flow with `scout.flow_ext.max_weight_matching` and a tighter upper
  bound that stays in 64-bit `Int`.
- `noi09p1.cp` (63 vs 120 raw / 88 stripped; `5102817.cpp`): lexicographically
  smallest perfect bipartite matching via leaf peeling and cycle orientation.
  Separates `Pos` and `Val` domains, uses Euclidean `%`, and fixes the
  original's infinite loop on unsolvable self-loop inputs.
- `coci25c3p3.cp` (43 vs 61 raw / 59 stripped; `7730378.cpp`): bipartite
  2-coloring (`scout.connectivity.two_color`), degree-1 peeling for the unique
  matching, and longest alternating-path DP.
- `cco16p3.cp` (200 vs 498 raw / 451 stripped; `5568296.cpp`): all five
  subtasks of CCO '16 P3 Legends — cycle chords via `std.dsu.Dsu`, cycle
  detection, DFS back-edge depth checks, degree check, and subtask 5's
  vertex-biconnected components (`scout.connectivity.biconnected_components`)
  replacing 100 lines of custom bridges/articulation/BCC traversal and unifying
  the duplicated 3-, 4-, and 5-cycle checks into one helper.
- `cco21p3.cp` (75 vs 148 raw / 68 stripped; `4552873.cpp`): Eulerian tour on
  a doubled tree with cyclic neighbor order. Replaces `__gnu_pbds::tree` (and
  an unused `Bit` in the original) with `std.fenwick.Fenwick.lower_bound`.
- `noi21p3.cp` (85 vs 379 raw / 163 stripped; `5669954.cpp`): directed graph
  connectivity — SCC condensation via `std.graph.scc`, transitive-reduction
  spanning tree via Kahn peeling, root-to-node vertex weights via
  `std.tree.Rooted.weighted`, and per-query `scout.connectivity.virtual_tree`
  plus forward/backward reachability (eliminating the original's custom
  `FastCin`, bump allocator, sparse table, Kosaraju SCC, and `Tree` classes).
- `noi16p2.cp` (99 vs 237 raw / 189 stripped; `4798070.cpp`): articulation
  points + sparse 2D grid complement connectivity (`@multitest`). Uses
  `std.graph.components` on the row-segment sweep graph and
  `scout.connectivity.articulation_points` on the radius-2 neighborhood.
- `btoi20p3.cp` (40 vs 157 raw / 76 stripped; `4656866.cpp`): offline odd-cycle
  (bipartite connectivity) queries via monotone divide-and-conquer and
  `std.dsu.ParityDsu` mark/undo_to.

Corpus coverage notes: (1) a full scan of `corpus/submissions/` shows that
`cmt21r2ztr` is the only explicit 2-SAT submission in the corpus and was
already ported by `scout_batch_9`; (2) no standalone general-graph Hierholzer
Eulerian circuit submission exists in the corpus, so `cco21p3` serves as the
cyclic Eulerian tour case, while `cco16p3` and `noi21p3` exercise vertex-
biconnected components and SCC condensation + virtual trees with zero overlap
against all earlier batches.
