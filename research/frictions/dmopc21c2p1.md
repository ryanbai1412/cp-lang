# Friction report: dmopc21c2p1

## The dream's ternary search is slower than the sweep

- **Where:** the whole port
- **What felt wrong:** the dream ternary-searches L with an O(N) cost per probe; the port keeps the sort-and-sweep, which is O(N log N) once and needs no convexity argument. No language gap; the dream is a different algorithm.
- **Goal it hurts:** none
- **Severity:** low

## Measurements

gen.py 1 1000000 (N = 1e6, now a max case in gen.py): port 0.18 s, accepted C++ 0.11 s.

## Dream

The committed `dream.cp` (ternary search) is unchanged.
