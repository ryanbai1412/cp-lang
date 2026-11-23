// expect: ok, output "6\n8\n10"
// A list literal given where `[D]T` is asked for and `D` is the struct's own
// domain parameter is a list, and its length sets `D` (R04.8.10), as passing
// a list variable does.
use std.fenwick.{Fenwick}
use std.beats.{BeatsSegTree}
var f = Fenwick([1, 2, 3])
print(f.all)
f.add(0, 5)
print(f.fold(0..2))
print(BeatsSegTree([5, 1, 4]).all_sum)
