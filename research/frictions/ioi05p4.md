# ioi05p4: Birthday

## Dream

```cp
let wish: [Seat]Seat = read()  // a permutation, generated as one
fn worst(order: [Seat]Seat) -> Int:
    let shifts = [(i.label - order[i].label) % n for i in Seat].sorted()
    (n - shifts.circular_gaps(n).max()! + 1) / 2
```

## Circular gaps are a window plus a wrap term
- **Where:** `wrap` beside `max(shifts[j] - shifts[j - 1] for j in 1..n)` (port
  lines 27-28); a ring of sorted points
  modulo n has one gap per point, which a `circular_gaps(period)` would say.
- **Severity:** low.

## A permutation input needs its own generator
- **Where:** `random_wish` exists only so `read` generates a permutation; a
  `Perm<Seat>` input type would generate and check it.
- **Severity:** low.

## Window slices keep the list's keys
- **Where:** port line 28 takes the gaps by index, `shifts[j] - shifts[j - 1] for j in 1..n`,
  rather than over `shifts.windows(2)`, because a window keeps the source list's
  keys, so a window from offset 1 has no key 0. Fresh `cpl run`:
  ```
  let xs = [1, 4, 9, 16]
  print(xs.windows(2).map(w => w[1] - w[0]).collect())
  ```
  ```
  error[R0101]: key 0 is not one of the view's keys 1..3: `.nth(k)` reads by position
  ```
- **General fix:** position-relative indexing on window views (`w[0]` = first),
  or a `pairs()`/`gaps()` adapter; owner: stdlib.
- **Severity:** low.
