# std.bag doc notes

Choices made where the spec was silent, and contradictions found, while
bringing `std/bag.cp` to the doc standard (`std/README.md`).

## Choices

- `SortedBag.first` / `last`: O(1) as their own docs say ("kept current by
  every update"). The struct doc listed them among the O(log n) operations
  and `lib/containers.md` 2.2 says O(log n); the struct doc now lists them
  with the O(1) ones.
- `SortedBag.add_many(x, c)`: traps when `c < 0`; `c == 0` does nothing.
- `nth(i)` on every container here: `None` for any `i` outside `0..len`,
  negative included, never a trap, matching the stated "or `None` if out of
  bounds".
- `take_first(ok)`: removes one copy; `None` and no change when `ok` holds
  nowhere; debug traps on non-monotone `ok` at the answer like
  `first_true`.
- `drain(r)` (generator): each copy yielded once, ascending; a loop that
  stops early leaves the rest in the bag.
- `drain(after:, before:, while:)` on `SortedBag`, `SortedSet`,
  `SortedMap`: bounds are exclusive, the first failing element stays, each
  copy is one element of the result, and giving both `after` and `before`
  traps.
- Cursors: `key_of`, `count_of`, `next_cursor`, `prev_cursor` on a stale
  cursor are unspecified in release; debug traps when the cursor's node
  slot is free. `BagCursor(id: Int)` carries no version, so a reused slot
  cannot be detected.
- `join(other)` on `SortedBag`/`SortedSet`/`SortedMap`: the precondition
  (strictly increasing across the seam) is checked in O(1) with the cached
  ends and traps when violated, rather than being left to debug builds.
- `SortedBag.absorb`: counts add. `SortedMap.absorb(other, join)`: shared
  keys give `join(self's, other's)`, or other's value without `join`,
  independent of which side is drained (same rule as `FlatMap.absorb`).
- `SortedBag.sum()`: cost written as O(n). The stated "O(1) when augmented,
  O(n) otherwise" leaves the cost open, and `BagNode` has no sum field, so
  the augmented case never applies.
- `AugSortedBag` / `AugSortedSet` folds: combine `leaf(k)` in ascending key
  order, one per copy, so `op` need not be commutative; empty selections
  and `lo >= hi` give `id`.
- `BucketCount`: writes outside `r` trap, as does a count going negative;
  reads outside `r` (`count`, `count_lt`) see 0 and never trap, so
  `count_lt` takes any `x`.

## Contradictions and gaps

- No `pub fn range(r)` exists on `SortedBag` or `SortedSet`, though both
  struct docs and decisions.md 103 make `range(r)` (a `SortedRange<K>`
  view) the replacement for `first_ge`/`first_gt`/`last_le`/`last_lt`/
  `count_range`. The private `range_*` helpers are there. Needs a stub (and
  a `SortedRange<K>` type) to be documented. (resolved: added `range(r: Range<K>) -> SortedRange<K>` to both and the `SortedRange` view, decision 168)
- `lib/containers.md` 2.2 still lists `first_ge`/`first_gt`/`last_le`/
  `last_lt`/`count_range`/`bsearch_first`/`bsearch_last` on `SortedBag`
  and `OrderedSet` in 44.1; decisions.md 103 removed the first five and
  117 renamed `first_where` to `bsearch_first`/`bsearch_last`. The stubs
  use `first_true`/`last_true`, matching `core/04-indices.md`; 117's
  `bsearch_*` names are stale. (resolved: decisions 103 and 137 win; containers.md 2.2 marked superseded)
- `SortedBag` has two methods named `drain`: `drain(r) -> yields K` and
  `drain(; after:, before:, while:) -> List<K>`. Overloading on labels
  with different result shapes goes against one spelling per job; one of
  them should be renamed. (resolved: renamed `drain(r)` to `drain_range(r)`, decision 166)
- `SortedSet.sum(keys)` promises O(log n) "subtree sums are kept", but
  `SortedSet` wraps a `SortedBag` whose nodes keep no sums, and
  `lib/containers.md` 5 defers a summed `SortedBag` until a second user.
  Kept as stated; the implementation must add the sum. (resolved: kept stub; the implementation keeps subtree sums)
- `SortedBag.sum()` returns `Int` whatever `K` is, while
  `SortedSet.sum` returns `K`. (resolved: returns `K`, decision 166)
- `BucketCount.kth(k)` is 1-based; every other order statistic in the
  library (`nth`) is 0-based. (resolved: 0-based, decision 166)
- `SortedBag`/`SortedSet` mutating generator `drain(r)`: see the
  `mut self` generator note in `heap.md`. (resolved: kept, as in heap.md)
