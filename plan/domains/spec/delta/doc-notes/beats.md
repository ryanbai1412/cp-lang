# beats doc notes

- `BeatsSegTree.new`, `HistoricBeatsSegTree.new`: the spec does not say what happens when both `a` and `fill` are given; chose a trap (two sources of initial values is a mistake), and zeros when neither is given (as stated).
- `BeatsSegTree.chdiv`: "floor division" kept; documented rounding toward negative infinity for negative values, since that is what floor means.
- `BeatsSegTree.chmod`: the spec says only `a[i] % m` with `m > 0`; chose the floored remainder in `0..m` for negative values, consistent with `chdiv`'s floor division. Core `spec/core/03-numbers-layout.md` does not pin `%` on negatives, so this needs a core rule.
- `BeatsSegTree.sum`, `min_val`, `max_val`, `get`: take `mut self` (they push tags); kept and noted in the `sum` doc, though it means queries cannot run on a shared borrow.
- `HistoricBeatsSegTree`: "historic sum of all versions" is unspecified; chose version 0 as the initial array and one new version per `chmin`, `chmax` or `add` call (even on an empty range), with `hist_sum(r)` summing every version's values over `r`.
- `HistoricBeatsSegTree.hist_max`: chose to include the initial values in the history.
- `HistoricBeatsSegTree` has no `assign`, `set`, `get` or `min_val`, unlike `BeatsSegTree`; left as is.
