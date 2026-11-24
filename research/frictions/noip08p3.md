# Friction Report: noip08p3

## 1. Rows as plain `Int`
- **Where:** `best(d: 0..=last, a: 0..n, b: 0..n)` and `kind[a2, d + 1 - a2]`.
- **What felt wrong:** the column is derived from the step and the row, so neither can be a domain value
  without converting back and forth; the bounds check `b2 < n and d + 1 - a2 < m` is by hand.
- **Goal it hurts:** types carry the proofs.
- **Ideal code:** `kind.at(a2, d + 1 - a2)` returning `?` and a diagonal walk helper.
- **General fix:** `[R, C]T.at(r: Int, c: Int) -> ?T` for computed coordinates.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5185022; max (50 x 50) 0.005 s vs C++ 0.021 s, same output. The generator keeps both sides >= 2: on a one-wide class the AC prints the single path's sum and the port 0.
