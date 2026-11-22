# prefix.cp doc notes

## Contradictions

- `Prefix` (whole struct): `lib/ops-ranges.md` builds `Prefix(op, a)` over any op with `fold(r)` (by difference for a `Group`, `prefix` only otherwise); the stub builds `prefix(a)` / `Prefix.new(a)` over `+` with `.sum(r)`. Documented the stub; the op-generic form is unresolved drift.
- `scan`: its prose mentions `pre.fold(r)`, but `Prefix` has no `fold` method, only `sum`. Left the sketch as written.
- `Prefix.filling` / `_Prefix2.filling` / `_PrefixList.filling`: they return `FillPrefix<R, C, T>` and `FillPrefix<T>`, but those shapes are declared as `_FillPrefix2` and `_FillPrefixList`. Signatures left unchanged.
- `suffix_fold` / `prefix_fold`: the old prose used `m.id`, `m.op`, `D.K`, `r.start` and `r.end`, but there is no `m` parameter (it is `op`) and ranges use `.l` / `.r`. Rewrote the prose as `op(pref[r.l], suff[r.r])` and "the identity", keeping the meaning.
- `ArithDiff.add_arith`: the old prose used `i.ord - r.start.ord`. Restated it as "the `k`-th position of `r`, from 0", which means the same.

## Choices

- `FillPrefix.set`, `_FillPrefix2.set`, `_FillPrefixList.set`: writing out of fill order traps. These are append-only tables, so a skipped cell would make later sums silently wrong.
- `FillPrefix.prefix` / `.sum` and the 2-D and list shapes: reading a cell that is not filled yet traps. With `Reverse`, this means `prefix(b)` works only once everything left of `b` is filled, and `sum(r)` works within the filled suffix.
- `Prefix.first_ge` / `first_gt`: ties go to the smallest cut, and `first_ge` with `target <= 0` gives the first cut. Negative values are an unchecked precondition, matching `Fenwick.first_ge`.
- `_PrefixList.prefix` / `.sum` / `_FillPrefixList.*`: a `k` outside `0..=n`, or a range outside `0..n`, traps. Plain `Int` indices are not domain-checked, so the check happens at run time.
- `_PrefixList.filling`: `n < 0` traps.
- `Prefix.all` and empty domains: an empty table sums to `0`. `scan` over an empty `D` gives one cut holding `from` or the identity.
- `SweepAdd.add`: the range must not start before the next position the cursor will advance to; otherwise it traps. This reads the stub's "at or ahead of the current sweep cursor" with the cursor meaning the next position.
- `SweepAdd.advance`: an `i` that is not the next position in fill order traps.
- `SweepAdd.current`: before the first `advance`, it is `0` rather than a trap, matching an all-zero accumulator.
- `prefix_excl`: a single position gets the identity.
