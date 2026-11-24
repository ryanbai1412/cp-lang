# Friction Report: noip09p3

## 1. Path min/max over a condensation by hand
- **Where:** the `low`/`high` passes over `condense(road, comp)`.
- **What felt wrong:** "the cheapest price on some walk from 1" is a DAG DP the condensation makes easy, but
  each pass is spelled as a loop with optional labels.
- **Goal it hurts:** express.
- **Ideal code:** `let low = dag.fold_from(comp[1], least, min)`.
- **General fix:** a `fold_from(source, init, combine)` DP on a topologically ordered DAG.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5185139; max (n = 1e5, m = 5e5) 0.091 s vs C++ 0.127 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let (domain C, comp) = scc(road)
let dag = condense(road, comp)
let least = C.fold(v => price[v], by: comp, min)
let most = C.fold(v => price[v], by: comp, max)
let low = dag.fold_from(comp[City.first!], least, min)
let high = dag.rev.fold_from(comp[City.last!], most, max)
print(max(high[c]! - low[c]! for c in C if low[c] != None and high[c] != None) ?? 0)
```
