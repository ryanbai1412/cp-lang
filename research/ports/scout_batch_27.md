# Scout batch 27

Ten graph submissions from the corpus, re-solved in CP Lang from the
problem statements: shortest paths over states, planar duals and rigid
pieces, a Kruskal-style merge tree, DSU, a functional graph and a DAG of
deadlines. This is a Stage-1 critique pass. Nothing in `proto/cpc/` or
`crates/` was changed for it. What the ports taught us is in
[`research/archive/frictions/friction-backlog-batch-27-graphs.md`](../../../research/archive/frictions/friction-backlog-batch-27-graphs.md).

Every port runs today on the reference interpreter (`proto/cp.py
lib-run`) with the real `stdlib/`, and `check.py` compares each one with
its original on random inputs, half small and half medium:

```
$ python3 proto/ports/scout_batch_27/check.py              # every port, 60 inputs each
$ python3 proto/ports/scout_batch_27/check.py ioi18p3 300  # one port, 300 inputs
```

All ten agree with their originals. `noi10p4` prints one of many valid
schedules, so `check.py` validates that line instead of comparing it. No
port was run at the problem's largest size or timed, and none was built
with `proto/cpc`: the ports use `std.graph` with adjacency lambdas, which
`proto/cpc` does not have.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

## The ports

Each port names its original on its `// original:` line.

Shortest paths:

- `coci20c6p2.cp`: Alias. Dijkstra per query over vertices named by words.
- `noi10p3.cp`: Altitude. A minimum cut of a planar grid, found as a
  shortest path over the blocks between the two outsides.
- `cco19p2.cp`: Sirtet. Rigid pieces fall; how far is a shortest path from
  the floor over "rests on" edges, with the pieces found by DSU.
- `noi07p1.cp`: Social Network. Floyd-Warshall that also counts shortest
  paths, for betweenness.

State spaces and functional graphs:

- `ioi11p1.cp`: Tropical Garden. States are (fountain, arrived by its best
  trail); BFS backwards over the one-successor map, then cycle lengths.
- `btoi18p1.cp`: Love Polygon. A matching on a functional graph, leaves
  first and then the cycles.

Kruskal and DSU:

- `ioi18p3.cp`: Werewolf. Two merge trees (Kruskal reconstruction trees by
  vertex order), subtree ranges, and an offline rectangle count.
- `acc7p3.cp`: Network Connections. Kruskal over neighbours in sorted
  order, starting from cables already laid.
- `noi15p1.cp`: Automated Program Analyzer. DSU over compressed variable
  numbers.

DAG:

- `noi10p4.cp`: Flight Control. Deadlines pushed back along a DAG in
  reverse topological order, then a greedy per plane.

## What the corpus did not have

The unported part of the corpus has no bridges or biconnected components
problem with an accepted submission, and no 0-1 BFS one, so neither is in
this batch. `noi14p2` (the usual incremental SPFA problem) has no accepted
submission to compare with, and `cco19p6` was already ported.

`noi07p1` is also in `proto/ports/scout_batch_23/`, which landed while
this batch was being written; the two ports were made independently.
