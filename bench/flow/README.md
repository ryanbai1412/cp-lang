# `std.flow` against matched C++

Each `*.cp` here calls one `std.flow` solver on a generated input; the `*.cpp` beside it
is the same algorithm written directly in C++ (same arc layout, same search order, same
generator; the Dinic-based ones share `_dinic.inc`), so the ratio measures the library and
the code generator, not the algorithm.

```sh
cargo build --release -p cpl-cli
bench/flow/run.sh               # every solver
bench/flow/run.sh costflow      # one
```

`run.sh` builds both with `g++ -std=c++17 -O2`, checks that their outputs agree, and
prints the best of five wall-clock runs.

| solver | input | CP Lang | C++ | ratio |
|---|---|---|---|---|
| `FlowGraph.max_flow` (Dinic) | 5000 vertices, 200000 arcs | 0.051 s | 0.039 s | 1.31 |
| `min_weight_matching` (Hungarian) | 1500 x 1500 | 0.197 s | 0.164 s | 1.20 |
| `max_matching` (Hopcroft-Karp) | 100000 + 100000, 400000 edges | 0.086 s | 0.075 s | 1.15 |
| `CostFlow.min_cost_flow` | 2002 vertices, 64000 arcs | 1.341 s | 1.181 s | 1.14 |
| `general_matching` (Edmonds) | 10000 vertices, 15000 edges | 0.052 s | 0.055 s | 0.95 |
| `max_weight_general_matching` (weighted blossoms) | 800 vertices, 160000 edges | 0.908 s | 1.022 s | 0.89 |
| `max_closure` (Dinic) | 50000 vertices, 300000 needs | 0.157 s | 0.141 s | 1.11 |
| `max_bounded_flow` (two Dinics) | 5000 vertices, 210000 arcs | 0.084 s | 0.072 s | 1.17 |
| `gomory_hu` (Gusfield) | 500 vertices, 5000 edges | 0.121 s | 0.116 s | 1.04 |
| `global_min_cut` (Stoer-Wagner) | 600 vertices, 20000 edges | 0.231 s | 0.228 s | 1.01 |

Where the gap comes from, measured by swapping one piece at a time:

- Arc tables are `List`s indexed by arc id. The hot loops (Dinic's BFS and path search,
  the Dijkstra and augmentation of `slope`, Hungarian, the blossom slack scans) run in
  `unchecked:`, since every arc id comes from `add`; without it min-cost flow is 1.41.
- `[V]?Int` distances cost at most 0.03 over an `Int.MAX` sentinel in the generated
  C++ for min-cost flow, within run-to-run noise; the library keeps the optionals.
