// expect: ok, output "1 2 3 4 5 1 1 1"
// reserve grows the backing lists of every pooled structure and leaves the contents alone.
use std.bag.{SortedBag, SortedSet, SortedMap, AugSortedBag}
use std.bitset.{XorTrie}
var b = SortedBag.of([3, 1])
b.reserve(50)
b.add(2)
var s = SortedSet.of([5])
s.reserve(10)
s.add(4)
var m: SortedMap<Int, Str> = SortedMap()
m.reserve(8)
m[2] = "x"
var a: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k)
a.reserve(16)
a.add(7)
var t = XorTrie(3)
t.reserve(20)
t.add(5)
print(b.to_list(), s.to_list(), m.len, a.len, t.len)
