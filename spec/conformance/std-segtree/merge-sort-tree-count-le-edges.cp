// expect: ok, output "0 0 5 2"
use std.segtree.{MergeSortTree}
let t = MergeSortTree([3, 1, 4, 1, 5])
print(t.count_le(2..2, 9), t.count_le(0..5, 0), t.count_le(0..5, 5), t.count_le(1..4, 3))
