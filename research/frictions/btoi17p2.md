# btoi17p2 friction

## No min-plus `Mat`, so the hop matrix and its product are hand-written

- **Where:** `spec.cp` `type Hop = [Slot, Slot]?Int`, `fn then(x: Hop, y: Hop)` (a k^3
  min-plus loop) and `Monoid(id: ..., op: then)` for `SegTree(hops, hop)`.
- **What felt wrong:** the dream says it in one line, `Mat<Slot, Slot>(over: Semiring(min, +))`
  and `SegTree(*, hop)`; today the product and its identity are a local helper.
- **Goal it hurts:** express, coherence (`Semiring(min, +)` exists and `Mat` documents
  `over:` and `SegTree(*, ...)` of matrices, but neither works).
- **Ideal code:** `corpus/problems/btoi17p2/dream.cp`.
- **Repro:** `var h = Mat<?Int>.filled(2, 2, fill: None, over: Semiring(min, +))` traps R0401
  "Mat over a non-default semiring is not supported yet"; and
  `SegTree(*, [Mat<Int>.identity(2), Mat<Int>.identity(2)])` is E0201 inside `Monoid.of`
  ("doesn't know the name `T`": `Mat` has no `T.one`, its identity needs a size).
- **General fix:** `Mat` keeps its semiring (and a size-free identity, or `Mat<R, C>` over
  domains so the identity is `[R, R]`), so `*`, `pow` and `SegTree(*, ...)` run over any
  `Semiring`; helps every min-plus/max-plus matrix port (path products, DP transitions).
- **Severity:** medium (worked around with a user `Monoid`).

## 1.67x slower than the accepted C++ on the largest input
- **Where:** the whole port; k = 5, n = 5e4, m = 2.5e5, q = 1e4.
- **What felt wrong:** `cpl build` C++ 0.05 s against the accepted C++'s 0.03 s (the
  measurement below), over the 1.1x line (idiomatic-code/R10). Not profiled.
- **Goal it hurts:** performance.
- **Ideal code:** the port as written.
- **General fix:** profile the `[Slot, Slot]?Int` table product (a fresh table per `mat_mul`,
  `?Int` cells) against the C++'s plain arrays; the fix belongs in how fixed-size `?Int`
  tables are built and copied, so every small-matrix port gains.
- **Severity:** low (20 ms absolute, 1000 ms limit).
- **Owner:** perf. Repro: the largest generated input, port against `orig.cpp`.

## Measurements

Largest input (k = 5, n = 5e4, m = 2.5e5, q = 1e4): accepted C++ 0.03 s; `cpl build` C++
0.05 s (23,793 bytes), interpreter 1.19 s; all three agree.

## Dream std::function

- `Mat(.., over: Semiring(min, +))` stores the semiring as fn fields (`Ring<T>.add/mul` in std/matrix.cp), so every Mat over a semiring builds std::functions; the dream folds `[Slot, Slot]?Int` tables with a top-level `chain` in a `Monoid` instead. Sent to the stdlib owner.
