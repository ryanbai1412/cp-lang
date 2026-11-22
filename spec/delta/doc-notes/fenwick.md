# fenwick doc notes

- Contradiction: `spec/lib/ops-ranges.md` (sections 8 and 10) spells the structure `Fenwick(op, a)` over any `Commutative` op with `fold(r)` (and a `Group` for range folds), while the stub is `Fenwick<D, T>(a)` over `+`/`-` with `sum(r)`. Documented the stub as written; one of the two needs to change.
- `Fenwick.new`, `DualFenwick.new`: chose that an empty `D` is allowed (all sums zero), as for every other domain-keyed container.
- `Fenwick.first_ge`, `first_gt`, `kth`: negative values are documented as unspecified results, not traps, since checking costs O(n); `target <= 0` gives the start cut (follows from "smallest `b` with `prefix(b) >= target`").
- `Fenwick.kth`: the spec is silent on `k < 1`; chose a trap, since `k` counts from 1 and `kth(0)` has no meaning (matches `windows` trapping on `k < 1`).
- `Fenwick.set`: chose "add the difference", as `spec/lib/ops-ranges.md` section 10 states for `s[i] = x` with a group.
- `SparseFenwick2D`: the phase rules are unstated; chose traps for `add_point` after `build`, `build` twice, and `add`/`sum` before `build`, and for `add` at an unregistered point (silently dropping it would give wrong sums); `sum` over unregistered or out-of-range coordinates contributes zero.
