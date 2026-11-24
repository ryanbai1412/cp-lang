// expect: ok, output "1 7 13 7 0 0\n4 8 9"
// rule: std.bag.AugSortedBag.fold std.bag.AugSortedSet.fold
// One `fold(keys)` takes any ordered range, as `range(r)` does on `SortedBag` (decision 103): `..b` is a prefix fold, `a..` a suffix fold, an empty range folds to `id`.
use std.bag.{AugSortedBag, AugSortedSet}
var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k)
for x in [3, 1, 3, 7]: b.add(x)
print(b.fold(..3), b.fold(..=3), b.fold(2..), b.fold(1..=3), b.fold(4..4), b.fold(5..2))
var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k)
for x in [1, 3, 5]: let _ = s.add(x)
print(s.fold(..5), s.fold(3..), s.fold(1..=5))
