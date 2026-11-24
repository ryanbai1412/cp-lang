# Friction report: cco20p5

## Dream

`dream.cp` (unchanged) keeps the offline timeline. The port goes further and is online:

```cp
var gaps = SegTree<X, Gaps>(Monoid(Gaps.none, Gaps.join))
for e in events:
    (if e.op == 'A': add else: remove)(e)
    gaps.set(e.l, at(e.l)); gaps.set(e.r, at(e.r))
    print(min(overlap_best(), (overlap: 0, span: gaps.all.span?)).span)
```

## Offline segment tree over time becomes online std structures

The v0.2 port packed each event into an `l * width + r` key, put every interval copy on
O(log Q) nodes of a hand segment tree over time, and walked it with hand Fenwick prefix-max
trees, three undo-log lists, pairs packed into one `Int` by `(x + b) * w + y` and `1e9` /
`pinf` sentinels. The rewrite answers online: std `SortedBag`s of `(r, l)`, `(l, r)` and the
lengths give the best overlapping pair and the shortest interval, and a std `SegTree` over
positions with a `Gaps` monoid (greatest l ending in the range, least r starting in it,
shortest disjoint span) gives the best disjoint pair. No packing, no undo, no sentinels:
missing values are `?Int`. O(Q log C).

## Remaining friction

- No `min` over `?Int` that skips `None` (`max` works because `None` sorts first), so the
  port writes a two-line `least`.
- The C++ is about 54 KB (W0201 over the 32 KB target): the `SegTree` and `SortedBag`
  templates are large. It still runs the max case in 1.7 s of the 3.5 s limit.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. Corpus tests (9) pass.
