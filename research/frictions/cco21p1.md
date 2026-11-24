# Friction report: cco21p1

## Dream

```cp
let at = a.buckets()
@memo fn before(x: Val, y: Val) -> Int = ...   // pairs with an x before a y
var inv = a.inversions()
for s in swaps:
    inv += 2 * before(order[s], order[s.next!]) - at[x].len * at[y].len
    order.swap(s, s.next!)
    print(inv)
```

## Hand bookkeeping becomes std

The v0.2 port read the bounds as bare `Int`s, filled `occ` by hand, counted the starting
inversions with a hand Fenwick loop and memoised on an `lo * k + hi` integer key with a
mutable default. The rewrite types every bound (`Slot = 1..k` for the swaps), takes `buckets`
and `inversions` from std, and memoises in a `Map<(Val, Val), Int>`.

## Remaining friction

- No `@memo`: the memo map and its two-line lookup are written by hand.
- A `Slot` is not a `Val`, so `Val(Int(s))` converts before `.next`.

## Verification

`gen.py` is new (max: n = 1e5, k = 300, q = 1e6). 30 random cases agree with the v0.2 port
and 30 with the accepted C++. Corpus tests pass; the max case runs in 0.27 s.
