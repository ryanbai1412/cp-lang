# Friction report: dmopc20c4p4

## The throws read one at a time

- **Where:** `for i in Throw: let behind: 0..=Int(i) = read()`
- **What felt wrong:** the dream reads the whole list with `read [read Int]Int` and walks `a.items`, but each `a_i` is bounded by its own index (`0 <= a_i < i`); a bound depending on the position has no list type, so the port reads inside the loop to keep the bound.
- **Goal it hurts:** ergonomic
- **Ideal code:** `let a: [Throw]0..=Throw.index = read()` (a per-index bound)
- **General fix:** types: a list element bound that may mention its index.
- **Severity:** low

## Measurements

gen.py 2 2000000 (n = 2e6): port 0.06 s, accepted C++ 0.20 s; they agree on 30 cases and the 8 tests.
