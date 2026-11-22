# std.deque doc notes

Choices made where the spec was silent, and contradictions found, while
bringing `spec/std/deque.cp` to the doc standard (`spec/std/README.md`).

- `Deque.last(k)`: `None` when `k >= len`, traps when `k < 0`. A negative
  offset is a bug, not an absence; `None` for "not that many elements"
  matches `first`/`last` on an empty deque.
- `Deque.rotate_left(k)` / `rotate_right(k)`: `k` is reduced with Euclidean
  `%` (decisions.md 1.9), so a negative `k` rotates the other way and
  `rotate_right(k)` is `rotate_left(-k)`. An empty deque does nothing for
  every `k` instead of dividing by zero. Matches decision 4.3's "signed
  `rotate_left`".
- `Deque.reserve(cap)`: a `cap` at or below the current capacity, including
  `cap <= 0`, does nothing and never traps.
- `Deque.clear` keeps the ring's capacity, so the stated O(1) holds.
- `Deque.pop_front_while` / `pop_while`: `ok` is called at most `k + 1`
  times and the first failing element stays. `pop_while` returns back
  first, as its existing doc says; the two orders are deliberately
  mirror images.
- `Deque.items` yields places `0..len` from the front (the `Int` keys of
  `q[i]`), not ring slots.
- `LinkedList.first` / `last` keep their stated trap on empty, unlike
  `Deque.first` / `last`, which return `?T`. Logged as an inconsistency
  between the two types (the spec states both); `LinkedList` should
  probably return `?T` too, with `!` at the call site.
- `LinkedList.pop_back` is an alias of `pop`; `Deque` has no `pop_back`.
  Two spellings for one job violates "one way per job"; left alone.
- The struct doc of `Deque` says the literal `var q: Deque<Int> = []`
  works; nothing in `spec/core` defines list literals converting to a
  library container. Examples use `Deque.new()` / `Deque.of(...)` instead.
