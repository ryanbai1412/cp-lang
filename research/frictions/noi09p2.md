# Friction Report: noi09p2

## 1. A monotone-decision queue replaced by `monge_paths`
- **Where:** `monge_paths(List(Sentence.cuts), cost: (j, i) => F64(line(j, i)) ** F64(p))`.
- **What felt wrong:** nothing; the old `wins` queue with `first_true` was a hand-rolled LARSCH.
- **Goal it hurts:** none.
- **Ideal code:** as written; `cuts.adjacent` walks the lines.
- **General fix:** none.
- **Severity:** none.

## Measurements
Optimal layouts are not unique, so `checker.py` checks the total, the order and the line costs.
Stress: 30/30 agree with the v0.2 port and with AC (C++ needs `ulimit -s unlimited`); max
(N = 1e5, P = 10) passes the checker both ways, 0.03 s vs C++ 0.35 s.
