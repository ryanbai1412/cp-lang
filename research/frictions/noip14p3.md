# Friction Report: noip14p3

## 1. Unbounded knapsack over a capped domain
- **Where:** `next[H.at(h + up) ?? H.last!] min= min(taps[h], next[h]) + 1`.
- **What felt wrong:** "clamp into the domain" is spelled `H.at(x) ?? H.last!`.
- **Goal it hurts:** express.
- **Ideal code:** `next[H.clamp(h + up)] min= ...`.
- **General fix:** `Domain.clamp(Int) -> D`.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 10000, m = 1000, k = 5000) 0.004 s vs C++ 0.027 s, same output.
