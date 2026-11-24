# Friction Report: noip06p3

## 1. Earliest free gap on a machine is a linear scan
- **Where:** `while any(busy[k, u] for u in t..t + d): t += 1`.
- **What felt wrong:** "the first gap of length d at or after t" is an interval-set query; the port marks
  time units in a bool grid.
- **Goal it hurts:** express.
- **Ideal code:** `let t = free[k].first_gap(d, from: ready[j])`; `free[k].remove(t..t + d)`.
- **General fix:** a `first_gap(len, from:)` query on std's interval set.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5165783; max (m = n = 20) 0.004 s vs C++ 0.003 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
for j in order:
    done[j] += 1
    let (k, d) = (machine[j, done[j]], dur[j, done[j]])
    let t = free[k].first_gap(d, from: ready[j])
    free[k].remove(t..t + d)
    ready[j] = t + d
print(max(ready)!)
```
