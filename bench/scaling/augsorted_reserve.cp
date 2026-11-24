// n: 512
// reserve moves the keys already there. At n = 512 the copy and the doubled block stay
// under 128 KB at 4n too, so malloc serves both from the heap and no size pays fresh-page
// faults that the other doesn't.
use std.bag.{AugSortedBag, AugSortedSet}
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
var eb: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k)
var fb: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k)
for x in xs:
    eb.add(x)
    fb.add(x)
for r in 0..32768:
    var e = eb.clone()
    e.reserve(2 * n) // timed: bag.AugSortedBag.reserve
    var f = fb.clone()
    f.reserve(2 * n) // timed: bag.AugSortedSet.reserve
    s += e.len + f.len
print(s)
