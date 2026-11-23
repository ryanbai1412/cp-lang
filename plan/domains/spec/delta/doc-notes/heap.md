# std.heap doc notes

Choices made where the spec was silent, and contradictions found, while
bringing `std/heap.cp` to the doc standard (`std/README.md`).

- Ties (module doc): no heap is stable. Among equal keys under `by:`, which
  element `peek` shows and `pop` returns is unspecified, but `peek` always
  shows what the next `pop` returns. A stable heap needs a sequence number
  per entry (more memory and a wider compare); the documented `(d, v)` /
  `(d, desc(v))` entry idiom gives a fixed order when one is wanted.
- `IndexedHeap`: ties between equal keys go to the smaller `v` in `D`'s
  order, so its pop order is fully determined. `D` is always ordered and
  the extra compare is cheap, so the determinism costs nothing worth
  avoiding; this differs from `Heap` on purpose.
- `IndexedHeap.decrease` / `increase`: an equal key is not an update and
  returns `false`; an absent `v` is inserted and returns `true`.
- `Heap.push_pop(x)`: returns `x` and leaves the heap unchanged when the
  heap is empty or `x`'s key is no greater than the top's (Python
  `heappushpop`).
- `Heap.replace_top(x)`: returns the old top even when `x` is smaller; on an
  empty heap it pushes `x` and returns `None`.
- `Heap.drain()` (a `mut self` generator): a loop that stops early leaves
  the elements not yet yielded in the heap. See the generator note below.
- `ErasableHeap.remove(x)` with `x` absent: release behaviour is
  unspecified; debug builds trap when they find the bad removal surfacing.
  `len` counts it out at once.
- `MeldHeap.add_all(delta)`: applies to the elements present now, not later
  pushes; `meld`/`absorb` keep each element's value as its own heap
  reported it. With `by:`, the key must keep its order under the shift.
- `Settle.get(k)`: the best priority offered so far, and for a settled key
  the one it popped with; `None` if never offered. Ties among equal
  priorities are unspecified, as for `Heap`. `best` must return one of its
  arguments (like `min`/`max`).
- `Heap.of`, `MinMaxHeap.of`, `MeldHeap.of` take no `by:`, so they always
  order by the elements themselves: there is no O(n) max-heap from a list
  except through `desc` entries. Contradiction with `lib/containers.md`
  2.5, which lists `MaxHeap.of(xs)`; since `MaxHeap` is now an alias of
  `Heap`, `MaxHeap.of` would build a min-heap. Probably wants
  `of(xs, by:)`. (resolved: `Heap.of(xs, by:)`, decision 166)
- Contradiction: decisions.md 15.8 says `MinHeap<T>()` takes no arguments
  and `by:` is an error, and the list of removed features (around
  "`Rev<T>` (use `Desc<T>`/`desc(x)`), `Heap<T>(by:)`") drops
  `Heap<T>(by:)`, and `lib/containers.md` 5 calls `MinHeap(by:)` resolved by
  the entry idiom; `lib/ops-ranges.md` (Heaps) and these stubs make
  `Heap<T>(by:)` the one heap type. The stubs follow ops-ranges (later,
  round 5). (resolved: kept stub `Heap<T>(by:)` over 15.8/18, recorded in decision 166)
- Contradiction: `lib/containers.md` 1.4 and 2.5 still describe
  `ErasableMinHeap`/`ErasableMaxHeap`/`IndexedMinHeap`/`IndexedMaxHeap`
  and `peek -> ?(key: D, val: T)` with named fields; the stubs have one
  type each and an unnamed `?(D, K)`. `IndexedHeap` has no `by:`, so a
  max-keyed indexed heap needs `Desc<K>` keys. (resolved: kept stub; containers.md 1.4 and 2.5 marked superseded)
- Contradiction: `Heap.drain` (and `SortedBag.drain(r)`) are `mut self`
  generators, while decisions.md (around "`cpl_ir::pushed::Feeds`
  intentionally does not push `-> yields` on `mut self` methods") says
  such generators are avoided because an early `break` could leave the
  container half-mutated. The docs define the early-stop state; the
  implementation must honour it or the signatures should return `List`. (resolved: kept: each element leaves before it is yielded, so an early `break` leaves a valid container, unlike `IntervalMap`'s partition)
