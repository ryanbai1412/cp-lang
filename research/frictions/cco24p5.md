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

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
use std.algebra.{MinCount}
domain End = 0..4
var bad = SegTree<End, MinCount<Int>>(MinCount.monoid(), fill: MinCount(0, 0), act: add)
print(bad.all.val)
```

```text
error[E0302]: `Action.of` (`std.algebra`) does not work on the types it was given here: expected `Action<MinCount<Int>, Int>`, found `Action<MinCount<Int>, MinCount<Int>>`
  --> probe.cp:3:11
3 | var bad = SegTree<End, MinCount<Int>>(MinCount.monoid(), fill: MinCount(0, 0), act: add)
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = note: std/algebra.cp:155: inside `Action.of`; the gap is in `std.algebra`, not in your code
  = note: std/segtree.cp:117: called from `SegTree.on_domain`
  = note: std/segtree.cp:93: called from `SegTree.new`
```

**Status:** open.
