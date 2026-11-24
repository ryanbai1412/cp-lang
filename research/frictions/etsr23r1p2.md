# etsr23r1p2: throwing out overlapping ranges

## Dream

A set of disjoint ranges that answers "remove every range overlapping
`l..=r`" in one call, e.g. `held.drain(overlapping: l..=r)` on a
`SortedMap<Int, Int>` read as start -> end. The port spells it as two
queries: a `drain(after:, while:)` over the starts in `l..=r` and a
`last_lt(l)` check for the one range that starts before `l`.

## `drain(after:)` is exclusive, so an inclusive start is `after: l - 1`
- **Where:** `held.drain(after: l - 1, while: (start, _) => start <= r)`.
  A `drain(l..=r)` taking a key range would say what it means.
- **Severity:** low.
