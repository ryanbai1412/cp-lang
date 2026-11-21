# Friction: geometry floating-point P0 audit

Audit of floating-point handling across `stdlib/geometry.cp`, the 291 scoreboard ports, and the P0 `floating-point` bug-replay suite (`5/6 = 83%`).

## 1. [Resolved] Bare `== 0.0` in `seg_dist` and squared-vs-linear `eps` in circle intersections

- **Area:** geometry
- **Goal:** hard-to-get-wrong
- **Severity:** medium
- **Status:** resolved
- **What happened:**
  1. `seg_dist(p, a, b)` checked `if n2 == 0.0:` with bare floating-point equality instead of an explicit `eps: F64 = 1e-9` guard (`n2 <= eps * eps`).
  2. `circle_line_intersect` compared `r^2 - d^2` against linear `eps` (`1e-9`) and `circle_intersect` compared `h^2` against linear `eps`, which breaks tangency at large radii (`r = 1e6`, where `r^2 = 1e12` and 1 ULP exceeds `1e-9`).
- **Resolution:** Added `eps: F64 = 1e-9` to `seg_dist` (`n2 <= eps * eps`), switched `circle_line_intersect` and `circle_intersect` to linear distance tolerances with `sqrt(max(0.0, ...))`, and added `test_float_eps_guards` in `stdlib/geometry.cp`.

## 2. [Resolved] `sqrt` of negative `F64` silently returns `NaN` (`flt_sqrt_neg_nan`, P0 bug-replay 6/6)

- **Area:** core
- **Goal:** hard-to-get-wrong
- **Severity:** medium
- **Status:** resolved
- **What happened:** In `research/scoreboard.json`, the only unhandled P0 `floating-point` bug replay (`5/6 = 83%`) was `flt_sqrt_neg_nan`: calling `sqrt(x)` on a tiny negative float (e.g. `r*r - d*d = -1e-15` from rounding in geometry) silently produced `NaN` instead of trapping in debug/checked mode, poisoning downstream comparisons and sorts.
- **Resolution:** Made `sqrt(x < 0.0)` trap with `sqrt(<x>) of a negative number` (exit 3) in `cpl-interp` and under `-DLOCAL` (`cpl build --checked`) via `cpl::fsqrt`, keeping release builds raw `std::sqrt` (decision 84). Verified with `sqrt_negative_traps_in_interp_and_checked_cpp_not_release`, `golden_math_assert_and_input_errors`, and `bench/goal_tests/bug_replay/flt_sqrt_neg_nan.cp` (now `caught_on_run`, `floating-point` 6/6 = 100%).

## 3. [Resolved] Unguarded `F64` range check in `cf2138f.cp`

- **Area:** geometry
- **Goal:** hard-to-get-wrong
- **Severity:** low
- **Status:** resolved
- **What happened:** `proto/ports/ecnerwala/cf2138f.cp` line 6 wrote `fn fits(a: Vec, b: Vec) -> Bool = 0.5 <= (a - b).len <= 1.0` without an explicit epsilon.
- **Resolution:** Added explicit `eps: F64 = 1e-9` safety margin (`fn fits(a: Vec, b: Vec, eps: F64 = 1e-9) -> Bool = 0.5 + eps <= (a - b).len <= 1.0 - eps`) so accepted points stay strictly inside `[0.5, 1.0]`. Verified with `proto/ports/ecnerwala/gate.sh cf2138f 50` (`PASS cf2138f (1 samples, 50 random cases)`).
