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
| `FlowGraph.max_flow` (Dinic) | 5000 vertices, 200000 arcs | 0.049 s | 0.041 s | 1.20 |
| `min_weight_matching` (Hungarian) | 1500 x 1500 | 0.196 s | 0.165 s | 1.19 |
| `max_matching` (Hopcroft-Karp) | 100000 + 100000, 400000 edges | 0.100 s | 0.079 s | 1.27 |
| `CostFlow.min_cost_flow` | 2002 vertices, 64000 arcs | 2.102 s | 1.160 s | 1.81 |
| `general_matching` (Edmonds) | 10000 vertices, 15000 edges | 0.047 s | 0.054 s | 0.87 |
| `max_closure` (Dinic) | 50000 vertices, 300000 needs | 0.169 s | 0.145 s | 1.17 |
| `max_bounded_flow` (two Dinics) | 5000 vertices, 210000 arcs | 0.092 s | 0.074 s | 1.24 |
| `gomory_hu` (Gusfield) | 500 vertices, 5000 edges | 0.128 s | 0.109 s | 1.17 |
| `global_min_cut` (Stoer-Wagner) | 600 vertices, 20000 edges | 0.201 s | 0.221 s | 0.91 |

Where the gap comes from, measured by swapping one piece at a time:

- `std.heap.MinHeap.pop` builds a two-element list per sift step; unrolling it takes
  `costflow` from 1.81 to 1.46.
- `[V]?Int` distances and `[V]?Arc` parents cost a further 0.2 over `Int.MAX` and
  arc-0 sentinels; the library keeps the optionals, since sentinels are a P0 bug class,
  and the fix belongs in the code generator.
- Arc tables are `List`s indexed by arc id, so every access is bounds-checked; the
  C++ indexes unchecked.
