// A BucketCount over 0..n holding n values, and SortedRange views of a SortedBag of n keys.
use std.bag.{BucketCount}
let n: Int = read()
var s = 0
for r in 0..1024:
    var e = BucketCount(0..n) // timed: bag.BucketCount.new
    e.add(r % n)
    e.clear() // timed: bag.BucketCount.clear
    s += e.len
var c = BucketCount(0..n)
for i in 0..n: c.add(i * 7919 % n)
for i in 0..64 * n:
    let k = i * 13 % n
    c.add(k) // timed: bag.BucketCount.add
    c.remove(k) // timed: bag.BucketCount.remove
    s += c.count(k) // timed: bag.BucketCount.count
    s += c.len // timed: bag.BucketCount.len
    if c.is_empty: s += 1 // timed: bag.BucketCount.is_empty
for i in 0..4 * n:
    let k = i * 13 % n
    s += c.count_lt(k) // timed: bag.BucketCount.count_lt
    s += c.kth(k) // timed: bag.BucketCount.kth
for r in 0..256:
    s += c.each().count() // timed: bag.BucketCount.each
let b = SortedBag<Int>.of([i * 7919 % n for i in 0..n])
for i in 0..4 * n:
    let k = i * 13 % n
    s += b.range(k..k + 8).last ?? 0 // timed: bag.SortedRange.last
    s += b.range(k..k + 8).len // timed: bag.SortedRange.len
    if b.range(k..k + 8).is_empty: s += 1 // timed: bag.SortedRange.is_empty
print(s)
