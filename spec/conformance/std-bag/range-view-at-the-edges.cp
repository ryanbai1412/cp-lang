// expect: ok, output "5 0 2 5 1\n7 1 -1 7 3 3\n2 4 0"
// rule: std.bag.SortedBag.range std.bag.SortedSet.range
// A range view at the edges: the whole range, an empty one, a single key with copies, an end at `Int.MAX`, and walking a view backwards.
let b = SortedBag.of([1, 3, 3, 7, Int.MAX])
print(b.range(..).len, b.range(5..2).len, b.range(3..=3).len, b.range(..=Int.MAX).len, b.range(Int.MAX..=Int.MAX).len)
print(b.range(7..).first!, b.range(..3).last!, b.range(8..=9).first ?? -1, b.range(2..8).rev.collect())
let s = SortedSet.of([2, 4])
print(s.range(..).len, s.range(4..=4).first!, s.range(..=1).last ?? 0)
