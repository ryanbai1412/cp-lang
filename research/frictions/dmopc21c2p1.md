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

## Port 1.6x the C++ at --max

- **Where:** the sort-and-sweep: `boss.sort()` and `for a in boss.rev:`.
- **What felt wrong:** 0.18 s against 0.11 s is over idiomatic-code/R10's 1.1x; the slower operation (the 1e6-element read, sort or sweep) is not isolated.
- **Goal it hurts:** the obvious way is the fast way
- **General fix:** profile the `--max` case against the C++ and route the hot operation to its owner.
- **Severity:** low (within the limit). **Owner:** perf.
