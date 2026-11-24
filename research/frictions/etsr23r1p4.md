# etsr23r1p4: Market (range add, floor division, min, sum)

The dream folds `each(lo: min, hi: max, sum: +)` in a generic `SegTree` and
states floor division as `apply_beats(stop, as_add)`. The port uses the
special-purpose `BeatsSegTree`, whose `chdiv` hides that rule.

## No generic beats update on `SegTree`
- **Where:** port line 13, `var t = BeatsSegTree(...)`, and line 17, `t.chdiv(l..=r, d)`;
  the dream's `t.apply_beats(r, stop: node => ..., as_add: node => ...)`.
- **What felt wrong:** the solution's one idea (a node whose min and max move
  together is just an add) is inside std, so a variant (ceil division, sqrt)
  needs a new `BeatsSegTree` method rather than one line.
- **Goal it hurts:** express, coherent stdlib (two segment trees).
- **Ideal code:** the dream's `SegTree(each(...), act: add)` with `apply_beats`.
- **General fix:** `SegTree.apply_beats(r, stop, tag)` on any lazy tree, with
  `BeatsSegTree` as a preset of it.
- **Severity:** medium.

## Enum variants with io tags
- **Where:** port lines 15-19, an `Int` op code and an `if/elif` chain; the
  dream's `enum Op: Add(r: Range<I>, x: Int) = 1, ...` does not parse: on a fresh
  `cpl`, `Add(x: Int) = 1` gives `error[E0107]: expected the end of the statement, found '='`.
- **General fix:** io tags on enum variants (also ioi01p1, ioi05p2).
- **Severity:** medium.

## Element bounds force a copy
- **Where:** port lines 12-13: the input is read as `[I]-2147483648..=2147483647`
  and copied to `[I]Int` with `[I] i => Int(a[i])` for `BeatsSegTree`.
- **General fix:** let `[D]R` widen to `[D]Int` where a range type `R` is
  accepted for `Int` (a view, not a copy).
- **Severity:** low.

## Tuned twin retired (2026-11-24)

The plain port is within 1.1x of the accepted C++ on `gen.py --max` on both counts (7 runs, `bench/v03_perf_bench.py --live`): 257.6 ms / 23.0 MB against 291.9 ms / 44.8 MB (0.88x time, 0.51x memory), since 87f0fbde0 keeps `BeatsSegTree`'s second extremes only once `chmin` or `chmax` runs and hands a constant node's value down as one tag, so `perf.cp` is deleted (performance/R6 keeps a twin only over 1.1x).
