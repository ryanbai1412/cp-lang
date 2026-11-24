# Friction report: egoi22p3

The v0.2 port hand-built the DFS forest (recursive `build_tree` with `vis`, `par`, `kids`) and climbed parents to spell out each path. The v0.3 port uses `Rooted.forest` over `g.only(...)` (the graph without vertex 1) and `forest.path(a, b)` for each paired path. The odd-degree test folds into the odd-component test. Reads are typed (`[m](u: V, v: V)`).

Gap: the grader is played over stdin with `Wire`, as the package has no local `@grader` stub, so the port cannot be the `social_engineering(n, edges)` function of the dream. Also, `open.chunks(2)` yields a trailing single chunk; pairs are indexed as `open[2 * i], open[2 * i + 1]`.

## Dream

```cp
for (a, b) in open.chunks_exact(2):
    route[a] = forest.path(a, b)
    route[b] = forest.path(b, a)
open.len.is_odd().then(open.last!)
```

## Measurements

Times include the Python interactor. A 2e5-vertex cycle (pair_up recursion 2e5 deep, 8 MB stack) took 1.25 s and 110 MB. gen.py seed 3 at size 2e5 (8864 Maria moves) took 7.7 s for the port and 8.0 s for the v0.2 port. The accepted C++ is grader-only (`GetMove` undefined), so it was not run.
