# Friction Report: noip10p3

## 1. Two-colouring by DSU with a foe pointer
- **Where:** `foe: [V]?V` beside `Dsu<V>`.
- **What felt wrong:** "these two must be apart" is a parity DSU; the port encodes parity with one
  representative of the other side.
- **Goal it hurts:** express.
- **Ideal code:** `var side = ParityDsu<V>()`; `if not side.apart(a, b): return print(anger)`.
- **General fix:** a parity/bipartite DSU in std.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and AC 5205348 on gen.py inputs; max (N = 2e4, M = 1e5) 0.017 s vs C++ 0.056 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let (n, m) = read<(2..=20000, 1..=100000)>()
domain V = 1..=n
let feuds = read<[0..m](a: V, b: V, anger: 1..=1e9)>()
var side = ParityDsu<V>()
for (a, b, anger) in feuds.sorted(by: f => desc(f.anger)):
    if not side.apart(a, b): return print(anger)
print(0)
```
