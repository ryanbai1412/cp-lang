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
| `FlowGraph.max_flow` (Dinic) | 5000 vertices, 200000 arcs | 0.049 s | 0.038 s | 1.29 |
| `min_weight_matching` (Hungarian) | 1500 x 1500 | 0.191 s | 0.153 s | 1.25 |
| `max_matching` (Hopcroft-Karp) | 100000 + 100000, 400000 edges | 0.087 s | 0.073 s | 1.19 |
| `CostFlow.min_cost_flow` | 2002 vertices, 64000 arcs | 1.619 s | 1.148 s | 1.41 |
| `general_matching` (Edmonds) | 10000 vertices, 15000 edges | 0.047 s | 0.049 s | 0.96 |
| `max_weight_general_matching` (weighted blossoms) | 800 vertices, 160000 edges | 1.241 s | 0.979 s | 1.27 |
| `max_closure` (Dinic) | 50000 vertices, 300000 needs | 0.163 s | 0.144 s | 1.13 |
| `max_bounded_flow` (two Dinics) | 5000 vertices, 210000 arcs | 0.089 s | 0.071 s | 1.25 |
| `gomory_hu` (Gusfield) | 500 vertices, 5000 edges | 0.119 s | 0.107 s | 1.11 |
| `global_min_cut` (Stoer-Wagner) | 600 vertices, 20000 edges | 0.218 s | 0.221 s | 0.99 |

Where the gap comes from, measured by swapping one piece at a time:

- `[V]?Int` distances and `[V]?Arc` parents cost a further 0.2 over `Int.MAX` and
  arc-0 sentinels; the library keeps the optionals, since sentinels are a P0 bug class,
  and the fix belongs in the code generator.
- Arc tables are `List`s indexed by arc id, so every access is bounds-checked; the
  C++ indexes unchecked.
