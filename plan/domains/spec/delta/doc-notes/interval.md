# interval doc notes

- `IntervalMap.replace`, `SparseIntervalMap.replace`: contradiction. `spec/lib/containers.md` §2.3 and decisions.md (IntervalMap `on_old`) reject a `yields` generator on `mut self` because an early `break` could leave the partition half-painted, and specify `assign(r, val, on_old:)`; the stub has `replace(...) -> yields` instead. Kept the stub and chose "the map is fully repainted when the call is made, whether or not the pieces are consumed", which removes the half-painted state the decision worried about. (resolved: decision 67 wins: `assign(r, val, on_old:)`, `replace` removed, decision 166)
- `IntervalMap.get`: the library prose still calls it `at`; the stub's `[rename]` is followed.
- `IntervalMap.assign`, `range`, `IntervalSet.add`/`remove`/`contains_range`, `SparseIntervalMap.*`: bounds unstated; chose empty ranges as no-ops and a trap when a non-empty range reaches outside the domain (`lo..hi` for the sparse map), matching index rules in `spec/core/04-indices.md`.
- `IntervalMap.replace`, `range`, `SparseIntervalMap.update`: chose that pieces are clipped to `r` (stated for `range`, inferred for `replace` from "overwritten piece") and that `update` calls `f` once per clipped piece in ascending order.
- `IntervalMap.first_where`: chose "positions right of the cut `from`" (so `(0..i).r` starts at `i`) and that `ok` is called once per run; the stub says "at or after `from`".
- `IntervalSet.contains_range`: chose `true` for an empty `r` (vacuous).
- `IntervalSet.next_gap`: chose `None` when everything from `from` to the end of `D` is covered; the stub's "dynamic MEX" does not say what happens at the end of a bounded domain.
- `IntervalSet.add`, `SparseIntervalSet.add`: chose that touching half-open runs (`1..3` and `3..5`) merge, so `runs` counts maximal runs.
- `SparseIntervalSet.next_gap`: returns `Int`, not `?Int`; always exists since the set is finite. Consistent, noted only because `IntervalSet.next_gap` returns `?D`.
- `SparseIntervalMap.new`: chose a trap on `lo > hi` and zero runs for `lo == hi`.
- `IntervalSet`, `SparseIntervalSet`, `SparseIntervalMap`: not described anywhere in `spec/lib/containers.md`; documented from the stubs alone.
