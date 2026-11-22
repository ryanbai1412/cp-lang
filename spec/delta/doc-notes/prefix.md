# Doc notes: `std.prefix`

- `scan` without `from`: chose a compile error when `f` is a plain closure with
  no identity; the stub says "`f`'s identity" but `f: (T, T) -> T` has none.
- `scan`: the result's `sum`/`get`/`suffix`/`cyclic_sum`/`first_*` subtract
  stored folds, so they mean range folds only for `f = +`; documented, not
  restricted. `all` is `prefix` at the last cut, so it includes `from`.
- Contradiction: core/07-sequences.md and the `scan` doc index the table as
  `pre[c]`, `pre[v]` and call `pre.fold(r)`; `Prefix` declares no index
  operator and no `fold` (it has `prefix(b)` and `sum(r)`). Examples use
  `prefix`/`sum`.
- Contradiction: lib/ops-ranges.md §7–8 builds `Prefix(op, a)` and `Diff(op)`
  with `fold(r)`/`apply(r, f)`; the stub has `Prefix.new(a)` (no op, `+`
  only), `prefix(a)`, `sum(r)` and `Diff.add(r, x)`.
- `Prefix.first_ge` / `first_gt`: chose to trap when any element is negative
  (one flag recorded at build), rather than leave a non-monotone search
  unspecified; ties go to the first cut. `target <= 0` (`< 0`) gives the first cut.
- `FillPrefix*.prefix` / `sum`: decision 114 says non-empty queries over
  unfilled cells trap; chose "cells" to mean the positions the query sums (the
  rectangle for 2-D), so a `Reverse` table answers `prefix(b)` only once every
  position left of `b` is written, and an empty query never traps.
- `_PrefixList.sum` / `_FillPrefixList.sum`: an empty `Range<Int>` returns zero
  wherever it lies; a non-empty one reaching outside `0..n` traps.
- `_PrefixList.filling`: traps on `n < 0`.
- `Diff.finish` / `ArithDiff.finish`: chose that `finish` leaves the structure
  usable (takes `self`, not `mut self`), so more adds may follow.
- `SweepAdd.add`: "at or ahead of the current sweep cursor" read as: no position
  of a non-empty `r` may already have been advanced to; else trap.
- `SweepAdd.advance`: traps unless `i` is the next position in fill order.
- `SweepAdd.current`: chose to trap before the first `advance` (there is no
  "most recently advanced position"); zero would hide a misuse.
- Vocabulary: `suffix_fold` writes `r.start` / `r.end` and `m.op`, `m.id` for
  the parameter named `op`; `ArithDiff.add_arith` writes `i.ord` and
  `r.start.ord`. Core uses `r.l` / `r.r` and `Int(v)`. Left the wording.
- Contradiction: decisions 41.3 and 42.4 say `prefix_fold` / `suffix_fold`
  return `[D.bounds]T`; the stub says `[D.cuts]T` (cuts is the current name).
