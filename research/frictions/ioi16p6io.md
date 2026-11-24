# Friction report: ioi16p6io

The stdin twin of ioi16p6, with the same dream. Its spec.cp still used the
`MonotoneHull` DP and printed 45 for 46 on a generated case (`8 18 5`, points
`2 1, 17 16, 8 7, 12 10, 7 5, 15 17, 2 4, 2 3`); the port uses ioi16p6's
`LiChao` workaround and agrees with the accepted C++ and the old port.

## `MonotoneHull` does not give ties to the smaller tag

- **Severity:** high (silent wrong answer)
- **Status:** open (routed to stdlib; see `research/frictions/ioi16p6.md`)
- **Goal it hurts:** hard to get wrong
- **Area:** std (`std/dp.cp` `MonotoneHull`)
- **Ideal code (`dream.cp`):** `hull.add(Line(...), tag: best.count + 1)` and
  `hull.best_next(x)!`, an O(1) amortized hull.
- **Workaround:** `LiChao<Photo, (val: Int, count: Int)>` over `0..=m`, whose
  key `(val, count)` breaks ties toward fewer photos. O(log m) per operation.
- **General fix:** as in ioi16p6: one tie mechanism for line structures, an
  ordered key from the line's `at`, so the hull can serve `aliens` again.

## Measurements

Contestant-written tokens: port 342, original 722 (`corpus/audit.py` count). Emitted C++: 26847 bytes (`cpl build`, judge `generic`).
