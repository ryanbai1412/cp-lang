// expect: ok, output "1 5\n5 9 12\n1 5 9\n5 5 9\n1"
// rule: std.bag.SortedSet.range
// `SortedSet.range` and `SortedBag.range` read a generic `Range<K>` with one open end
// (`..b`, `a..`, `..=b`).
use std.bag.{SortedSet, SortedBag}

var s = SortedSet<Int>()
for x in [1, 5, 9, 12]: s.add(x)
print(s.range(..9).collect())
print(s.range(5..).collect())
print(s.range(..=9).collect())
var b = SortedBag<Int>()
for x in [1, 5, 5, 9]: b.add(x)
print(b.range(5..).collect())
print(b.range(..5).collect())
