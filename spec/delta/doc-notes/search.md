# search: doc-standard notes

- `bsearch_real`: the result within the tolerance was unpinned. Chose the final `hi` of the bisection (so `ok` holds at the result), with `ok(hi)` tested first; matches the stated `steps + 1` calls.
- `bsearch_real`, `ternary_*_real`: chose to trap on `lo > hi` or `steps < 0` rather than return a meaningless point.
- `ternary_max` / `ternary_min`: plateau ties go to the first position of the plateau (library default); golden-section can keep it by discarding only the right part on equal probes.
- `ternary_*` (discrete): a non-unimodal `f` returns an unspecified position of `r` rather than trapping, since detecting it costs O(n).
- `ternary_*_real`: the returned point was unpinned. Chose the midpoint of the final bracket.
- `parallel_bsearch`: an empty `r` gives `None` for every query; `ok` is not consulted before the first event, so a query already true with no events answers `r.start` (the first `t`). Traps on `queries < 0`.
- `gallop_first`: pinned the probe schedule `from`, `from + 1`, `from + 3`, `from + 7`, ... then bisection; only call counts depend on it.
- `dinkelbach`: pinned the stopping rule (`num - l * den <= eps` returns `l`) and chose to trap when `solve` reports `den <= 0`, the stated precondition.
- `split_best`: "best first" pinned as decreasing `T`; equal values come out in group order, then left piece before right piece; empty pieces are never queued and `best` is never called on an empty range.
- `split_best` (contradiction): the signature has no `T: Ord` bound but the ordering needs one; left unchanged. (resolved: stub now bounds `T: Ord`)
- `split_best` (contradiction): the old prose said the k best cost "O(k log(n + k)) `best` calls"; it is O(n + k) `best` calls (one per group, two per taken value) and O((n + k) log(n + k)) queue work. Documented the latter. (resolved: doc kept (O(n + k) `best` calls, O((n + k) log(n + k)) queue work))
