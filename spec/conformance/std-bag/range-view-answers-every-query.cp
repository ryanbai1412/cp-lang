// expect: ok, output "3 2 3 3 7\n3 3 3 1 1\n1\n4"
// rule: std.bag.SortedBag.range std.bag.SortedSet.range
// `range(r)` is one O(1) view in place of first_ge, last_le and count_range (decision 103): first, last, len, is_empty, each, rev and `for` all read through it.
let b = SortedBag.of([1, 3, 3, 7])
print(b.range(2..=7).len, b.range(3..=3).len, b.range(2..).each().collect())
let s = SortedSet.of([1, 3, 7])
print(s.range(2..).first!, s.range(..=6).last!, s.range(..7).rev.collect(), s.range(2..7).len)
print(Int(SortedSet.of([1, 7]).range(2..7).is_empty))
var t = 0
for x in s.range(1..4): t += x
print(t)
