# ioi12p6: Jousting Tournament

## Dream

```cp
var line = OrderedSet(Pos)           // the groups still in, by leftmost position
for round in Round:
    let (s, e) = read()
    let merged = line.remove_range(s..=e)  // order-statistics range removal
    ...
var strongest: [Node]?Rank = None     // max over an empty span is None
print(Pos.argmax(by: p => wins[leaf(p)])!)
```

## The k-th survivor is a Fenwick descent
- **Where:** `alive.first_ge(s + 1)!.left!`; an order-statistics set with
  `nth` and range removal is what the round says.
- **Severity:** medium.

## Max over an empty span needs -1
- **Where:** `top` and `inner` are `-1..n` with -1 as the identity of `max`;
  an optional-aware `max=` would let them be `?Rank`.
- **Severity:** low.

## The accepted C++ disagrees with a brute force
- **Where:** submission 4848321 under a sample grader differs from `brute.py`
  (seat the late knight everywhere and replay) on 6 of 300 generated cases,
  e.g. `7 3 3 / 4 1 0 5 2 6 / 5 6 / 1 2 / 2 3` (brute and port: 1, C++: 0).
  The port is stressed against `brute.py` and the old port.
- **Severity:** low.

## The accepted C++ assumes the rounds end with one knight
- **Where:** `orig.cpp`. Submission 4848321 walks only round `c - 1`'s tree, so
  when the rounds stop with several knights left it misses every other tree
  (`7 3 3 / 4 1 0 5 2 6 / 5 6 / 1 2 / 2 3`: answer 1, it prints 0). `orig.cpp`
  walks every survivor's tree; it then agrees with `brute.py` and the port on 500
  generated cases.
- **Severity:** medium (evidence only).
