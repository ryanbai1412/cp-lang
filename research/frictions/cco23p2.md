# cco23p2 — Real Mountains (CCO 2023 P2)

Port: `corpus/problems/cco23p2/port.cp`. The old port built the prefix and suffix maxima with
two hand loops, kept `prev_h: ?Int` across the level sweep, held raw `Int` columns in a
`SortedBag<Int>` with `V(...)` conversions, and spelled the successor sweep twice. The
rewrite reads the caps off `h.scan(max)` and `h.rev.scan(max)`, buckets columns by
compressed level, keeps the columns still below their cap in a `SortedSet<Col>`, and runs one
`least_above` sweep in each direction.

## Frictions

- **No `.adjacent` over a domain in the backends.** `for (l, u) in Level.adjacent:` checks,
  but `cpl run`/`build` stop with E0501, so the sweep is `for l in Level:` with
  `let Some(u) = l.next else: break`.
- **`SortedSet` has no `first_gt`.** `SortedMap` has one; on a set the successor is
  `seen.range(q.x + 1..).first!`.
- **A domain needs `.collect()` to be passed as an order.** `least_above` takes
  `Col.collect()` and `Col.rev.collect()` (1e6 each) because there is no type for "a walk over
  `Col`" a function can take.

## Dream

```cp
for (l, u) in Level.adjacent:
    ...
fn least_above(order: Seq<Col>, asks: [Col]List<(x: Int, times: Int)>) -> Z:
    ...
        for q in asks[c]: sum += Z(q.times) * Z(seen.first_gt(q.x)!)
print(cost + least_above(Col, from_left) + least_above(Col.rev, from_right))
```

## Measurements

Max case (1e6 random heights up to 1e9, limit 5 s): port 3.15 s / 351 MB, accepted C++
2.58 s / 232 MB, same output.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
var seen = SortedSet<Int>()
seen.add(3)
print(seen.first_gt(1)!)
```

```text
error[E0203]: `SortedSet.first_gt` was replaced by `range(1 + 1..).first`
  --> probe.cp:3:7
3 | print(seen.first_gt(1)!)
  |       ^^^^^^^^^^^^^^^^
  = help: write `range(1 + 1..).first` (or run `cpl fix` to migrate)
```

**Status:** not a gap: std spells a set successor `range(x + 1..).first` by design, as the port does.

```cp
let n: 1..=5 = read()
domain Col = 0..n
fn walk(order: Seq<Col>) -> Int = order.count()
print(walk(Col) + walk(Col.rev))
```

```text
6
```

**Status:** fixed: a function takes `Seq<Col>` and is passed `Col` or `Col.rev`, so `least_above` needs no `.collect()`.
