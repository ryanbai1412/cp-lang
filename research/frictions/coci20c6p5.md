# coci20c6p5 frictions

## Dream

```cp
for (a, window) in mo(asks, enter:, leave:): answer[a] = h
```

## Mo's algorithm by hand

std has no Mo's order or window driver, so the port sorts the asks by block and moves the two
ends itself, with `Paper.nth` for the offsets.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let a = [1, 2, 3]
var s = 0
print(mo([0..2, 1..3], add: i => s += a[i], remove: i => s -= a[i], answer: _ => s))
```

```text
3 5
```

**Status:** not a gap: `std.mo` has `mo(queries, add:, remove:, answer:)` and `mo_order`; the port's hand-rolled order and window walk are owed.
