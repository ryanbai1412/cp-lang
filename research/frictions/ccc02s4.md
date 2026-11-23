# Friction: `ccc02s4`

## An optional open-range start is reported as an unbounded-range gap
- **Where:** `spec.cp` `(c.next..).take(m)` (twice); `c.next` is `?P.cuts` (R04.3.2).
- **What felt wrong:** E0501 "`cpl run` and `cpl build` can't handle an unbounded range value yet" sent the fix toward the compiler; the real fix was `c.next!`. Repro: `domain P = 0..3; let c = P.cuts.first!; print((c.next..).take(2).map(e => Int(e)).sum())`.
- **Goal it hurts:** evidence
- **Ideal code:** the error names the `?P.cuts` start and offers `c.next!..`, as a `for` over the same range already does (E0302).
- **General fix:** run the optional-range-ends check before the E0501 lowering gate, for open-ended ranges too. Routed to diagnostics.
- **Severity:** low

## `c.shift(1..=m)` is specified but not implemented
- **Where:** `spec.cp` group ends `(c.next!..).take(m)`.
- **What felt wrong:** "the cuts 1..=m after `c`, clipped" is exactly R04.3 `c.shift(1..=m)`, but it gives E0203 (already in compiler's worklog from cf2124f2).
- **Goal it hurts:** express
- **Ideal code:** `c.shift(1..=m).map(e => cost(c, e)).min()!`
- **General fix:** implement `shift` on every domain position and cut.
- **Severity:** low

## Measurements
- `cpl-conform corpus ccc02s4`: 1/1 pass (8/8 tests). Agrees with the AC submission (g++ -O2) on 60/60 `gen.py` inputs (seeds 1..=60).
