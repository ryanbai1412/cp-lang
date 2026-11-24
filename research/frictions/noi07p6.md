# Friction Report: noi07p6

## 1. Top two of the child labels
- **Where:** `children(u, p).map(...).max_k(2).sorted(by: x => desc(x))`.
- **What felt wrong:** `max_k` leaves its order unspecified, so taking the largest and the second
  largest needs a sort of two items; the first version read `top[0]` as the largest and was wrong.
- **Goal it hurts:** correct-by-default.
- **Ideal code:** `let (a, b) = xs.top2()` or `max_k` returning largest first.
- **General fix:** give `max_k` a fixed order, largest first.
- **Severity:** medium (silent wrong answer).

## Measurements
Stress: 30/30 agree with the v0.2 port under `checker.py`. The AC C++ prints a fixed
non-optimal plan (S = 2n - 1, which DMOJ accepts), so it is no reference for S; the max plan
(n = 1000) passes the checker against the v0.2 port's S. 0.09 s.
