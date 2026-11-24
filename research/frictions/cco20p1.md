# Friction Report: cco20p1

## A sweep over a coverage count is written by hand
- **Where:** the `events` list, `sort`, and the `(watching, from)` loop.
- **What felt wrong:** "how many points of L..=R lie under exactly c intervals" is a standard
  sweep, but it takes an event list, a sort and a running pair, plus the tail after the last event.
- **Ideal:** `coverage(intervals, over: l..=r) -> [0..=n]Int` (or an interval-count map in
  `std.interval`) that returns the length covered by each count.

## Dream
```cp
use std.interval.{coverage}
let n: 1..=1e5 = read()
let l: -1e9..=1e9 = read()
let r: l..=1e9 = read()
let y: 1..=1e6 = read()
let friends: [n](x: l..=r, v: 1..=100, h: 1..=100) = read()
let views = [f.x - (f.h * y - 1) / f.v ..= f.x + (f.h * y - 1) / f.v for f in friends]
let seen_by = coverage(views, over: l..=r, counts: 0..=n)
for total in prefix_fold(+, seen_by)[1..]: print(total)
```

- Tokens: port 288, original 594
- Max input (`gen.py --max`): port 0.02 s, 12200 KB, original 0.02 s, 7404 KB
