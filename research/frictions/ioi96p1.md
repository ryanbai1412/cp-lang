# ioi96p1 — A Game

The mover's lead on a range is the larger of "take an end, minus the
opponent's lead on what is left". spec.cp fills `[I.ranges]Int` bottom-up
(the `interval_dp_by_length` idiom); the top-down form the recurrence reads
as does not check.

## `@memo(array)` rejects a `V.ranges` parameter
- **Where:** the top-down form of spec.cp:
  ```cp
  @memo(array)
  fn lead(r: I.ranges) -> Int:
      if r.len == 1:
          return a[r.first]
      max(a[r.first] - lead(r.drop(1)), a[r.last] - lead(r.drop_last(1)))
  ```
- **What felt wrong:** spec/lib/search-dp.md says `V.ranges` is a domain
  (a triangle of n(n+1)/2 slots) and `@memo(array)` takes any bounded
  parameter, but `cpl check` gives E0311 "`r: Range<I>` is unbounded": the
  parameter type decays to `Range<I>`. The port falls back to the
  bottom-up table.
- **Goal it hurts:** express, coherence.
- **Ideal code:** the snippet above.
- **General fix:** keep `V.ranges` as the bounded domain in parameter
  position, so `@memo(array)` stores the same triangle as `[V.ranges]T`.
  Helps every top-down interval DP. Routed to the compiler lane.
- **Severity:** low (the bottom-up table works).

## Measurement
- Emitted C++: 10658 bytes (`cpl build`, judge `generic`).
- Agrees with the AC submission on 10 tests and 300 generated inputs
  (interpreter on 100, C++ build on 200).
