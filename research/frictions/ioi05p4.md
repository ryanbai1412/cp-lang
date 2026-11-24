# ioi05p4: Birthday

## Dream

```cp
let wish: [Seat]Seat = read()  // a permutation, generated as one
fn worst(order: [Seat]Seat) -> Int:
    let shifts = [(i.label - order[i].label) % n for i in Seat].sorted()
    (n - shifts.circular_gaps(n).max()! + 1) / 2
```

## Circular gaps are a window plus a wrap term
- **Where:** `wrap` beside `shifts.windows(2)`; a ring of sorted points
  modulo n has one gap per point, which a `circular_gaps(period)` would say.
- **Severity:** low.

## A permutation input needs its own generator
- **Where:** `random_wish` exists only so `read` generates a permutation; a
  `Perm<Seat>` input type would generate and check it.
- **Severity:** low.

## Window slices keep the list's keys
- **Where:** `w.last! - w.first!`: `w[0]`, `w[1]` index by the source list's
  keys, so a window from offset 1 has no key 0. Position-relative access on a
  window would read as written.
- **Severity:** low.
