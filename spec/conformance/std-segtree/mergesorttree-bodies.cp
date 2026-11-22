// expect: ok, output "2 2 3 none 1 none"
use std.segtree.{MergeSortTree}
domain P = 0..5
let xs: [P]Int = [3, 1, 4, 1, 5]
let t = MergeSortTree<P, Int>(xs)
print(t.count_gt(P(0)..=P(4), 3), t.count_ge(P(0)..=P(4), 4), t.first_ge(P(0)..P(3), 2), t.first_ge(P(0)..P(3), 5), t.last_le(P(2)..=P(4), 3), t.last_le(P(2)..=P(4), 0))
