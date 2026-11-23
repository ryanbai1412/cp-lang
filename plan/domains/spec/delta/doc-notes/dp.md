# dp doc notes

- `LiChao.new`: chose a trap on an empty `over` (there is no coordinate to query).
- `LiChao.at`, `LiChao.line_at`: chose a trap when `x` is outside `over` rather than `None`; `None` is reserved for "no line covers `x`".
- `LiChao.line_at`, `LineContainer.line_at`: tie rule left unspecified among lines with equal value at `x`. A deterministic rule (say, first added) would force a different tree descent; the documented way to break ties is a `K` with a second field. Flagged as a determinism gap.
- `LiChao.add`: chose a trap when `on` reaches outside `over` (domains are preserved, not clipped); an empty `on` adds nothing.
- `monotone_minima`, `smawk`: chose `rows == 0` gives `[]`; `monotone_minima` with `cols == 0` gives `None` per row; `smawk` traps on `cols == 0` with rows, because its rows return a bare value.
- `knuth`: contradiction. `spec/lib/search-dp.md` writes `knuth(V, w)` over an index space; the stub is `knuth(n, cost) -> List<List<Int>>`. Followed the stub. Chose the table to be `(n + 1) x (n + 1)` with 0 for `j <= i`, and a trap on `n < 0`. (resolved: kept stub; search-dp.md table fixed)
- `aliens`: chose traps on `k < 0`, an empty `lambdas`, and a `lambdas` that never reaches a count of at most `k`. Documented the answer as `val - λ * k` at the least `λ` whose count is at most `k`.
- `subset_sums`: chose to keep repeated sums (one per subset), so the result has exactly `2^n` entries.
- `monge_path`, `monge_paths`, `MongePath`: chose a trap on empty `nodes`, and total `0` for a single node (`K` must have a `0`; the stub's generic `K` does not say so).
- `monge_path`, `monge_paths`, `MongePath.next`: the stub's tie rule ("the fewest edges is not guaranteed") leaves the predecessor among equal totals implementation-defined; kept it, flagged as a determinism gap.
- `MongePath.next`, `MongePath.val`: chose traps after the last node and for unreached `i`; `val(0)` is 0.
- `MonotoneHull.add`: chose equal slopes as allowed (keeping the better intercept); only a slope that goes back traps.
- `MonotoneHull.best_next`, `at_next`: chose a trap when `x` decreases; the pointer is shared by both methods and reset by `clear`.
- `MonotoneHull.len`, `LineContainer.len`: chose "lines kept", not "lines added", since dominated lines are dropped.
- `supermasks`, `masks_of_size`: chose traps on negative `bits`/`k` and on `m` outside `0..2^bits`; `masks_of_size` with `k > bits` yields nothing.
- `subset_folds`: chose increasing index order inside each mask, so non-commutative monoids are well defined.
- `backtrack`: chose preorder with `start` at depth 0 and children in `next`'s order; states are not deduplicated.
- `Step.prune`: chose that it affects only the step just yielded, before the body continues.
