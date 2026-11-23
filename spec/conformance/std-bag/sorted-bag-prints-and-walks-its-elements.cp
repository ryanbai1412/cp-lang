// expect: ok, output "1 3 3\n1\n3\n3\n3 3 1\n2 5"
// A SortedBag prints its elements in order, duplicates included, and `for`
// walks them (std-api/R12); a SortedSet prints its keys.
use std.bag.{SortedBag, SortedSet}
var b = SortedBag<Int>()
b.add(3)
b.add(1)
b.add(3)
print(b)
for x in b:
    print(x)
print(List(b.each_rev()))
print(SortedSet.of([5, 2, 5]))
