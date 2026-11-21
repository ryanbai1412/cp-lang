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
the network. Over the 12 rows the ratio's median is 1.16 and its maximum 1.47
(`general_matching`).

| solver | input | CP Lang | C++ | ratio |
|---|---|---|---|---|
| `FlowGraph.max_flow` (Dinic) | 5000 vertices, 200000 arcs | 0.070 s | 0.057 s | 1.23 |
| `FlowGraph.push_relabel` | 5000 vertices, 200000 arcs | 0.059 s | 0.049 s | 1.20 |
| `FlowGraph.push_relabel` | cf1404e network, 400 x 400 all `#` | 0.100 s | 0.092 s | 1.09 |
| `min_weight_matching` (Hungarian) | 1500 x 1500 | 0.393 s | 0.310 s | 1.27 |
| `max_matching` (Hopcroft-Karp) | 100000 + 100000, 400000 edges | 0.153 s | 0.122 s | 1.25 |
| `CostFlow.min_cost_flow` | 2002 vertices, 64000 arcs | 1.791 s | 1.727 s | 1.04 |
| `general_matching` (Edmonds) | 10000 vertices, 15000 edges | 0.085 s | 0.058 s | 1.47 |
| `max_weight_general_matching` (weighted blossoms) | 800 vertices, 160000 edges | 1.496 s | 1.469 s | 1.02 |
| `max_closure` (Dinic) | 50000 vertices, 300000 needs | 0.279 s | 0.250 s | 1.12 |
| `max_bounded_flow` (two Dinics) | 5000 vertices, 210000 arcs | 0.132 s | 0.108 s | 1.22 |
| `gomory_hu` (Gusfield) | 500 vertices, 5000 edges | 0.161 s | 0.153 s | 1.05 |
| `global_min_cut` (Stoer-Wagner) | 600 vertices, 20000 edges | 0.465 s | 0.422 s | 1.10 |

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
