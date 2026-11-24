# btoi20p3 — Joker

Dream: double the streets into a ring and ask `street.ring.minimal(state: undoing(ParityDsu<V>(),
add: ...), ok: d => not d.state.consistent())` for the shortest odd window starting after each
street; a query is YES when that window fits in the streets left open. Port: the editorial's divide
and conquer on `l` over a hand-written parity DSU with union by size and rollback.

## The port hand-writes a rollback parity DSU
- **Where:** port `var up`, `sz`, `par`, three parallel history lists `hist_child`, `hist_parent`,
  `hist_sz`, `find_root`, `add_edge`, `rollback(h, prev_odd)`; dream `undoing(ParityDsu<V>(), ...)`.
- **What:** about 45 lines restate std's parity DSU and its undo log, with the log split over three
  lists that must stay the same length (R3, R4). The dream's `undoing` wrapper is the same structure.
- **Why the port keeps it:** measured on `gen.py --max` (n = m = q = 2e5), the dream runs in 0.108 s
  against `orig.cpp`'s 0.186 s but peaks at 18.1 MB against 15.7 MB (1.16x, over the 1.1x RSS bar).
  The port runs in 0.18-0.19 s at 13.3 MB (1.04x time, 0.85x RSS). Same output on all three.
- **General fix:** the dream's extra memory is the doubled ring (`2m` window starts) and the
  per-start `?window` answers that `ring.minimal` materialises; a `minimal` that yields each start's
  answer lazily (or stores only the end index) would bring the dream under the bar, and the port
  would then be the dream.
- **Severity:** medium (a 45-line hand structure stays in a port).
- **Owner:** stdlib (`ring.minimal` storage), perf.

## `limit` uses `m + 1` for "never bipartite"
- **Where:** `var limit: [E]Int = [E] _ => m + 1` and `var opt = m + 1`.
- **What:** a sentinel for "no right end keeps it bipartite" (R5); `?Int` (or a `Cut<E>`) would say
  it. Owed rewrite. Owner: idioms.

## Reads and output, fixed with this report
The port read `(n, m, q): (Int, Int, Int)` with no bounds and printed `if ...: "YES" else: "NO"`.
It now reads `1..=2e5` each (the statement's limits, stated in `// Bounds:`) and prints `yes(...)`.

## The tuned twin is gone
`perf.cp` (the accepted C++'s keep-left recursion) was added when the port measured over 1.1x. The
port now measures 1.04x time and 0.85x RSS against `orig.cpp`, so performance-checklist R6 no longer
needs it; it is deleted with this report.
