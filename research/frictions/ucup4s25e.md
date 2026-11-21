# Friction: ucup4s25e (UCup 4 Stage 25 Zagreb E)

Port: `proto/ports/ecnerwala/ucup4s25e.cp`
Gate: `proto/ports/ecnerwala/gate.sh ucup4s25e 50` passed (`0 samples, 50 random cases`).
Metrics: 379 -> 302 CP tokens (`0.12x` raw, `0.37x` stripped); emitted C++ 13,699 -> 14,976 B (+1,277 B from pulling in `hull_extreme`/`better_dir`/`dot128`/`dot`, 23% of the 64 KB judge limit).

## 1. [Resolved] Offline query sort and hand two-pointer replaced by `hull_extreme`

- **Area:** geometry
- **Goal:** express
- **Severity:** low
- **What happened:** The initial port sorted queries offline and walked the upper hull with a hand-written `hi_idx` two-pointer loop, plus a global-mutating `try_max` helper.
- **Resolution:** Reversed the upper hull in place (`hull.reverse()`) so it forms a CCW convex polygon and answered every query directly in O(log n) with `hull_extreme(hull, Pt(v, 1))`, cutting 77 tokens.

## 2. `hull_extreme` requires reversing an `upper_hull`

- **Area:** geometry
- **Goal:** express
- **Severity:** low
- **What happened:** `upper_hull` returns vertices left-to-right (CW along the top), while `hull_extreme` expects a CCW polygon. Callers querying an upper hull must call `hull.reverse()` before `hull_extreme`.
- **Suggestion:** Keep `hull_extreme` CCW-only (one convention across all polygon functions), and document the `upper_hull` + `reverse()` pattern rather than adding an `_upper` twin.
