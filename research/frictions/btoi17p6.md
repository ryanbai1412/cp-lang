# Friction report: btoi17p6

Rethought from dream.cp. The v0.2 port joined each child into its parent with a reverse label scan over `Kid`. The v0.3 port builds `Rooted.from_parents([V] v => Kid.at(v.ord).map(k => parent[k]))` and folds with `tree.up(Monoid<Pick>(..), lift:, finish:)`, as the dream does. The parents still read as `[Kid]V` over `domain Kid = 1..n`.

No language gap.

## 2.5x slower than the accepted C++ at --max
- **Where:** the whole port, `gen.py --max`.
- **What felt wrong:** port 0.05 s against the accepted C++'s 0.02 s with an unlimited stack
  (the measurement below), over the 1.1x line (idiomatic-code/R10). Not profiled.
- **Goal it hurts:** performance.
- **Ideal code:** the port as written.
- **General fix:** profile `Rooted.from_parents` and `tree.up` (the per-node `lift`/`finish`
  closures and the `Pick` tuples) against the C++'s single recursion; the fix belongs in
  `Rooted.up`, so every subtree-fold port gains.
- **Severity:** low (30 ms absolute, 1000 ms limit).
- **Owner:** perf. Repro: `gen.py --max | port` against `orig.cpp` with `ulimit -s unlimited`.

## Measurements

gen.py --max: port 0.05 s. The accepted C++ recurses and dies (signal 11) under an 8 MB stack; with an unlimited stack it takes 0.02 s and gives the same output.
