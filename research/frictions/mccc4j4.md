# mccc4j4 frictions

## grid rows view is unimplemented
- **Where:** `print(Group.map(g => groups[g].sorted(by: t => (desc(pts[t]), t))[k - 1]))`.
- **What felt wrong:** the row index space needs its own `domain Group = 0..(n / m)` only to walk the rows; `groups.rows` (R04b.1.12) is E0203 "a `[n / m, m]Team` has no method `rows`".
- **Goal it hurts:** express, coherence (spec promises it).
- **Ideal code:** `print(groups.rows.map(g => g.sorted(by: t => (desc(pts[t]), t))[k - 1]))` on `read<[n / m, m]Team>()`.
- **General fix:** implement `g.rows`/`g.cols` views for every `[R, C]T` (stdlib lane; `spec/conformance/core-04/grid-rows-cols-views.cp` already a known gap in SUBSET.md). Helps every grid/group port.
- **Severity:** low (named domain workaround is one line).

## Dream copies

- `g.sorted(by:)` on a row lent by `groups.map` copied the row implicitly; the dream writes `List(g).sorted(by:)` so the one copy sorting needs is visible.
