# Friction report: ucup5s05b (Universal Cup 5 Stage 5 B)

Rewrote `proto/ports/ecnerwala/ucup5s05b.cp` to adopt `std.geometry.{Pt, dot}` and prelude `.sign`, replacing hand-rolled `fn sign`, six coordinate pairs (`(sx, sy)`, `(ex, ey)`, `(dx, dy)`, `(ux, uy)`, `(vx, vy)`, `(x0, y0)`), and manual dot products (`ux * a + uy * b + c`, `sign(vx - ux) * a + sign(vy - uy) * b`).

- Gate: `proto/ports/ecnerwala/gate.sh ucup5s05b 50` passed (`PASS ucup5s05b (0 samples, 50 random cases)`).
- Emitted C++: `12,686 -> 13,299 B` (`+613 B`, `+4.8%`, 21% of 64 KB judge limit; under the 5% ratchet without rebaselining after pruning unused `cpl_tie()` comparison templates).

## Generic `cpl_tie()` struct comparison templates were emitted even when no struct is compared

- **Severity:** low
- **Status:** fixed (`crates/cpl-codegen/src/emit.rs` now gates `TIE_OPS` on whether any struct/enum is actually compared, sorted, or used as a `Set`/`Map` key; `ucup5s05b` dropped from `14,169 B` to `13,299 B`).
- **Goal it hurts:** dependable-submission
- **Area:** codegen
- **What happened:** Adopting `Pt` in a previously struct-free port triggered all 6 generic `operator==/!=/</<=/>/>=` templates (`870 B`) even though `ucup5s05b` never compares `Pt` values.
- **General fix:** Gate the `cpl_tie()` operator templates in `crates/cpl-codegen` on whether any struct or enum is actually compared, ordered, or keyed in a set/map.

## No interval-union length in the stdlib

- **Goal it hurts:** express
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s05b.cp:24-31,74`
- **Actual:** the port sorts its half-open spans and sweeps them in a local `union_len`.
- **General fix:** an interval-union operation over a list of ranges (`spans.union()` returning disjoint
  ranges, with `.len` summing them), the shape behind every "covered length" or "distinct points" count.

## No per-axis sign on `Pt`

- **Goal it hurts:** express
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s05b.cp:33`
- **Actual:** a local `signs` builds `Pt(p.x.sign, p.y.sign)` for the unit step along a diagonal.
- **General fix:** `Pt` lifts the scalar `.sign` (and `abs`) componentwise, as it already does `+`, `-` and `*`.

## Measurement

Query coefficients stay `Int` (their statement bounds are not recorded); `t` is `0..`. Emitted C++
13,138 -> 14,322 bytes; work per query is unchanged.

