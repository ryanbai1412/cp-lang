// n: 65536
// SortedBag, SortedSet and SortedMap reserve: each copies n keys into room for 2n. At the default n the
// growth swung from 7.7x to 8.9x between runs as fresh allocations fault in pages.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let bag = SortedBag<Int>.of(xs)
let set = SortedSet<Int>.of(xs)
var map: SortedMap<Int, Int> = SortedMap()
for x in xs: map.set(x, x)
var s = 0
for r in 0..160:
    var f = bag.clone()
    f.reserve(2 * n)  // timed: bag.SortedBag.reserve
    s += f.len
for r in 0..160:
    var g = set.clone()
    g.reserve(2 * n)  // timed: bag.SortedSet.reserve
    s += g.len
for r in 0..160:
    var h = map.clone()
    h.reserve(2 * n)  // timed: bag.SortedMap.reserve
    s += h.len
print(s)
