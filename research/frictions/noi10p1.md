# Friction Report: noi10p1

## 1. Multiples of g as a stepped range
- **Where:** `sum(pairs[x] for x in (2 * g..=k).step(g))`.
- **What felt wrong:** nothing; the old `var x = 2 * g; while x <= k` walk is one range.
- **Goal it hurts:** none.
- **Ideal code:** as written.
- **General fix:** none; a lint could flag `while x <= k: ...; x += g` as `(a..=k).step(g)`.
- **Severity:** none.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = m = 1e5) 0.018 s vs C++ 0.040 s, same output.
