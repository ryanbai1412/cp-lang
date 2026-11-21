# Friction: geometry floating-point P0 audit

Audit of floating-point handling across `stdlib/geometry.cp`, the 291 scoreboard ports, and the P0 `floating-point` bug-replay suite (`5/6 = 83%`).

## 1. [Resolved] Bare `== 0.0` in `seg_dist` and squared-vs-linear `eps` in circle intersections

- **Area:** geometry
- **Goal:** hard-to-get-wrong
- **Severity:** medium
- **What happened:**
  1. `seg_dist(p, a, b)` checked `if n2 == 0.0:` with bare floating-point equality instead of an explicit `eps: F64 = 1e-9` guard (`n2 <= eps * eps`).
  2. `circle_line_intersect` compared `r^2 - d^2` against linear `eps` (`1e-9`) and `circle_intersect` compared `h^2` against linear `eps`, which breaks tangency at large radii (`r = 1e6`, where `r^2 = 1e12` and 1 ULP exceeds `1e-9`).
- **Resolution:** Added `eps: F64 = 1e-9` to `seg_dist` (`n2 <= eps * eps`), switched `circle_line_intersect` and `circle_intersect` to linear distance tolerances with `sqrt(max(0.0, ...))`, and added `test_float_eps_guards` in `stdlib/geometry.cp`.

## 2. `sqrt` of negative `F64` silently returns `NaN` (`flt_sqrt_neg_nan`, P0 bug-replay 5/6)

- **Area:** core
- **Goal:** hard-to-get-wrong
- **Severity:** medium
- **What happened:** In `research/scoreboard.json`, the only unhandled P0 `floating-point` bug replay (`5/6 = 83%`) is `flt_sqrt_neg_nan`: calling `sqrt(x)` on a tiny negative float (e.g. `r*r - d*d = -1e-15` from rounding in geometry) silently produces `NaN` instead of trapping in debug/checked mode, poisoning downstream comparisons and sorts.
- **Suggestion:** Trap on `NaN`-producing `sqrt(x < 0.0)` / `acos` / `asin` outside `[-1.0, 1.0]` in `cpl run` and `cpl build --checked` (or provide a checked/clamped math note in diagnostics), matching the existing NaN-free contract for `F64` comparisons.

## 3. Unguarded `F64` range check in `cf2138f.cp`

- **Area:** geometry
- **Goal:** hard-to-get-wrong
- **Severity:** low
- **What happened:** `proto/ports/ecnerwala/cf2138f.cp` line 6 writes `fn fits(a: Vec, b: Vec) -> Bool = 0.5 <= (a - b).len <= 1.0` without an explicit epsilon. It passes because the grid search finds a strictly interior point well inside `(0.5, 1.0)`, and the other 4 `F64` scoreboard ports (`cf2115c`, `cf2219c`, `cf2066e`, `cf2187e`) do pure probability/DP arithmetic or `sqrt` without geometry predicates.
- **Suggestion:** Keep exact integer `Pt` predicates (`turn`, `cross128`, `dot128`, `on_segment`, `segments_meet`, `arg`) as the default path; where `Vec`/`F64` predicates are unavoidable, pass an explicit `eps` via `close`/`sign_eps`/`vturn`.
