# cco24p5 frictions

## Dream

```cp
var bad = [Side] _ => SegTree<End, MinCount<Int>>(MinCount.monoid(), fill: MinCount(0, 0), act: add)
```

## Stock `add` on a MinCount tree

The `MaxCount`/`MinCount` docs show `SegTree(MaxCount.monoid, a, act: add)`, but `Action.of(add, …)`
over `MinCount<Int>` fails ("expected `Int`, found `MinCount<Int>`"). The port spells the range-add
action by hand: `Action<MinCount<Int>, Int>.new(apply: (d, v, _) => MinCount(v.val + d, v.count), …)`.

## Weighted minimum count

The count of a `MinCount` leaf here is a dp weight, not a multiplicity; the sum stays below
\(5 \cdot 10^{11}\), so it is reduced mod 1000003 only when read. A weighted min-count summary
would name that directly.
