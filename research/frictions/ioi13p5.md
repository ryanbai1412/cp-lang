# ioi13p5: Robots

## Dream

```cp
let (a, b, t): (0..=5e4, 0..=5e4, 1..=1e6) = read()
var weak: [a]Limit = read().sorted()
...
let first_weak = [Toy] y => weak.partition_point(w => w > toys[y].weight).right
let arrivals = toys.group_by(first_weak)            // [?Weak]List<Size>
fn done_in(m: Int) -> Bool:
    var left = MaxHeap<Size>()
    for r in Weak: left.push_all(arrivals[Some(r)]); left.pop_n(m)
    left.push_all(arrivals[None])
    small.sorted(by: desc).all(lim => left.pop_n(m).all(s => s < lim)) and left.is_empty
print(Minutes.first_true(m => done_in(m)) ?? -1)
```

## Sorting a keyed array forgets its keys
- **Where:** `weak.sort()` in place: `weak.sorted()` is a `List<Limit>`, so its
  `partition_point` gives a bare `Int` cut, not `Cut<Weak>` with `.right`.
  Sorting in place keeps `[Weak]` but reuses the input index space for ranks.
- **Severity:** medium.

## No `group_by` into a keyed array of lists
- **Where:** the `newly`/`heavy` loop: "bucket each toy by its first weak
  robot, or none" is three lines of push.
- **Severity:** low.

## A heap pops one at a time
- **Where:** `for _ in 0..minutes: left.pop()` and the small-robot loop; a
  `pop_n(k)` would read as the statement does ("each robot takes m toys").
- **Severity:** low.

## `?? -1` across an optional position
- **Where:** the final `if let m ... else: print(-1)`; `first_true(..) ?? -1`
  does not unify `?Minutes` with `Int`.
- **Severity:** low.

## The judge's function task has no CP Lang form
- **Where:** `orig.cpp` appends a main to submission 4688418's `putaway`.
- **Severity:** high.

## `Heap(by: desc)` is five times slower than `Heap<Desc<Int>>`
- **Where:** `MinHeap<Desc<Int>>` and the `desc(..)` pushes: the `by:` key
  compiles to a `std::function` called on every comparison (7.7s at the max
  case against 1.0s). The port stores `Desc` values instead of a key. Repro
  sent to Core.
- **Severity:** high.

## `drain().take(k)` empties the heap
- **Where:** the small-robot pop loop: lint L0588 suggests
  `for s in left.drain().take(minutes)`, but the generator runs to the end and
  drops the rest (answers go wrong). Repro sent to Core.
- **Severity:** high.
