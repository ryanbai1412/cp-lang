# Friction report: coi07p1

Already at the bar: a single greedy sweep over the typed positions, with no index arithmetic. Only header lines were added (limits, `// Bounds:`).

## Gap

K and the positions are reals, so they read as `F64` with no range type: F64 values have no range types.

## Measurements

gen.py --max (n = 1e5): port 0.18 s, accepted C++ 0.03 s, same output; the limit is 600 ms.

R11 run (fresh `cpl run`, `let k: 0.0..=1e9 = read()`):

```
error[E0302]: expected `Int`, found `F64`
  --> f.cp:1:8
  |
1 | let k: 0.0..=1e9 = read()
  |        ^^^
```

## Port 6x the C++ at --max

- **Where:** the sweep `for x in xs:` over `[0..n]F64` (and the `F64` reads).
- **What felt wrong:** 0.18 s against 0.03 s is over idiomatic-code/R10's 1.1x; with one O(n) sweep the time is likely reading 1e5 reals, but that is not measured yet.
- **Goal it hurts:** the obvious way is the fast way
- **General fix:** profile the `--max` case; if `F64` parsing dominates, make the reader's real parse as fast as its integer parse.
- **Severity:** low (within the 600 ms limit). **Owner:** perf.
