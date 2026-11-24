# Friction Report: noip18p3

## 1. Chains passed up through a table
- **Where:** `var chains = [Junction] v => List<Int>()` and the `tree.at.rev` loop.
- **What felt wrong:** "each child returns its spare chain" is a post-order fold; the port collects
  children's chains in a per-vertex list because a recursive fn over 5e4 depth is not safe to write.
- **Goal it hurts:** express.
- **Ideal code:** `tree.fold_up(v => ...)` passing each child's result to the parent.
- **General fix:** a bottom-up fold on `Rooted` that hands a vertex its children's results.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 4002963; max (n = 5e4) 0.14 s vs C++ 0.12 s, same output. The v0.2 skip-pointer union-find is now `SortedBag.take_first`.
