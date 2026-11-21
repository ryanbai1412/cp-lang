use std.bag.SortedSet

var b = SortedBag<Int>()
for x in [5, 2, 5, 1, 2, 5, 9, -3]:
    let _ = b.add(x)

print(b.range(2..).first ?? -99, b.range(3..).first ?? -99, b.range(10..).first ?? -99)
print(b.range(..=5).last ?? -99, b.range(..5).last ?? -99, b.range(..=-5).last ?? -99)
print(b.range(2..6).len, b.range(2..=5).len, b.range(6..2).len, b.range(6..2).is_empty)

var fwd: List<Int> = []
for x in b.range(2..=5): fwd.push(x)
print(fwd)

var rev: List<Int> = []
for x in b.range(2..=5).rev: rev.push(x)
print(rev)

print(b.range(2..=5).sum())

let (before, after) = (b.range(..5), b.range(5..))
print(before.len, before.first ?? -99, before.last ?? -99, after.len, after.first ?? -99, after.last ?? -99)

domain V = 0..8
var s = SortedSet<V>()
for i in [1, 3, 5, 6]:
    let _ = s.add(V.nth(i))
let r = V.nth(2)..V.nth(6)
let (lo_v, hi_v) = (s.range(..r.start), s.range(r.end..))
print(lo_v.len, lo_v.first!.to_idx, hi_v.len, hi_v.last!.to_idx, s.range(r).len)
print([x.to_idx for x in s.range(r)])
print([x.to_idx for x in s.range(r).rev])
