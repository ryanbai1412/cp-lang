# `std.flow` against matched C++

Each `*.cp` here calls one `std.flow` solver on a generated input; the `*.cpp` beside it
is the same algorithm written directly in C++ (same arc layout, same search order, same
generator; the Dinic-based ones share `_dinic.inc`, the push-relabel ones
`_pushrelabel.inc`), so the ratio measures the library and the code generator, not the
algorithm.

```sh
cargo build --release -p cpl-cli
bench/flow/run.sh               # every solver
bench/flow/run.sh costflow      # one
```

`run.sh` builds both with `g++ -std=c++17 -O2`, checks that their outputs agree, and
prints the best of five wall-clock runs. Times include generating the input and building
the network. Over the 12 rows the ratio's median is 1.10 and its maximum 1.26
(Hungarian).

| solver | input | CP Lang | C++ | ratio |
|---|---|---|---|---|
| `FlowGraph.max_flow` (Dinic) | 5000 vertices, 200000 arcs | 0.066 s | 0.057 s | 1.16 |
| `FlowGraph.push_relabel` | 5000 vertices, 200000 arcs | 0.053 s | 0.046 s | 1.15 |
| `FlowGraph.push_relabel` | cf1404e network, 400 x 400 all `#` | 0.098 s | 0.089 s | 1.10 |
| `min_weight_matching` (Hungarian) | 1500 x 1500 | 0.380 s | 0.301 s | 1.26 |
| `max_matching` (Hopcroft-Karp) | 100000 + 100000, 400000 edges | 0.137 s | 0.112 s | 1.22 |
| `CostFlow.min_cost_flow` | 2002 vertices, 64000 arcs | 1.730 s | 1.694 s | 1.02 |
| `general_matching` (Edmonds) | 10000 vertices, 15000 edges | 0.052 s | 0.055 s | 0.95 |
| `max_weight_general_matching` (weighted blossoms) | 800 vertices, 160000 edges | 1.476 s | 1.419 s | 1.04 |
| `max_closure` (Dinic) | 50000 vertices, 300000 needs | 0.275 s | 0.254 s | 1.08 |
| `max_bounded_flow` (two Dinics) | 5000 vertices, 210000 arcs | 0.126 s | 0.106 s | 1.19 |
| `gomory_hu` (Gusfield) | 500 vertices, 5000 edges | 0.158 s | 0.147 s | 1.07 |
| `global_min_cut` (Stoer-Wagner) | 600 vertices, 20000 edges | 0.453 s | 0.414 s | 1.09 |

Where the gap comes from, measured by swapping one piece at a time:

- Arc tables are `List`s indexed by arc id. The hot loops (Dinic's BFS and path search,
  the Dijkstra and augmentation of `slope`, Hungarian, the blossom slack scans) run in
  `unchecked:`, since every arc id comes from `add`; without it min-cost flow is 1.41.
- `[V]?Int` distances cost at most 0.03 over an `Int.MAX` sentinel in the generated
  C++ for min-cost flow, within run-to-run noise; the library keeps the optionals.
- `push_relabel`'s solve alone is 1.02x on the grid and 1.06x on the random network; the
  rest is building the graph. Its 2V buckets were `[List<V>() for _ in 0..2 * n]`, whose
  generated push_back loop and copy took 17 ms of a 57 ms solve; `List.filled` fixed that.
- Arc ids are `0..2^31` (decision 120), so arc handles are 4 bytes as in the C++. With
  `Int` ids Dinic was 1.50 and push-relabel 1.31.
- Heights as 4-byte ints instead of `Int` changed push-relabel by under 1%, within noise.
- `dinic.cpp` keeps `cap0` beside `lft`, as KACTL keeps `oc`: `FlowGraph` stores the
  initial capacities for `flow(a)`, `cap(a)` and `reset`. Against C++ without it Dinic is
  1.28 (75.8 ms vs 59.0 ms). The rest of its gap is building the network, not the solve:
  see `research/frictions/bench-flow.md`.
- `general_matching`'s search ran with bounds checks; in `unchecked:` it went from 1.47
  to 0.95.
