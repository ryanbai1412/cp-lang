// absorb drains the smaller map into the larger, so absorbing a big map into a small one costs
// the small one's size
use std.hashmap.{FlatMap, Counter}
let n: Int = read()
var s = 0
let xs = [i * 7919 % n for i in 0..n]
var big = Counter.of(xs)
var fb = FlatMap(0)
for x in xs: fb.set(x, x)
for _ in 0..1 << 15:
    var small = Counter.of([1, 2, 3, 4])
    small.absorb(mut big) // timed: hashmap.Counter.absorb [fixed: self]
    s += small.len
    big = small.take()
    var fs = FlatMap(0)
    for i in 0..4: fs.set(i, i)
    fs.absorb(mut fb, join: (x, y) => x + y) // timed: hashmap.FlatMap.absorb [fixed: self]
    s += fs.len
    fb = fs.take()
print(s)
