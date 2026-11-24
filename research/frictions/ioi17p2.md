# Friction report: ioi17p2

## Dream

```cp
let points = merge_by(red, blue, key: x => x)       // tagged with the colour they came from
domain P = 0..n + m
let near = [P] p => points.nearest(p, where: q => colour[q] != colour[p]).dist
var cost: [P.cuts]Int = 0
var latest = Map<Int, Cut<P>>(0 => P.cuts.first!)
for p in P:
    cost[p.r] = cost[p.l] + near[p]
    if let c = latest[bal[p.r]]: cost[p.r] min= cost[c] + abs(signed[p.r] - signed[c])
    latest[bal[p.r]] = p.r
```

## Merging two sorted lists goes through a sort

- **Where:** `var points = [...] + [...]` then `points.sort()`.
- **What happened:** std has no linear merge of two sorted sequences; the port concatenates and
  sorts, O((n + m) log(n + m)) for an O(n + m) step, and re-indexes the result into `P` by hand
  (`[P] p => points[Int(p)]`).
- **General fix:** `merge(a, b, by:)` in std, and `[P]T` construction straight from a list of
  length `|P|`.
- **Severity:** low.

## Nearest other-colour point needs two hand scans

- **Where:** the `left` / `right` scans with `seen_red` / `seen_blue`, and
  `min(gaps[p].0 ?? gaps[p].1!, gaps[p].1 ?? gaps[p].0!)`.
- **What happened:** "the closest earlier and later point satisfying a predicate" is the
  previous/next-match scan written twice, and the min of two optionals where at least one is
  present has no spelling (`min` over `?Int` values).
- **General fix:** `prev_match` / `next_match` (nearest index to each side with a predicate)
  and `min_some(a, b)` for optionals.
- **Goal it hurts:** express. **Severity:** low.

## Measurements

Contestant-written tokens: port 448, original 671 (`corpus/audit.py` count). Emitted C++: 16644 bytes (`cpl build`, judge `generic`).
The accepted C++ is the grader function `min_total_length` with no main, so the max input is timed only by `corpus/generators.py`.
