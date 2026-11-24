// expect: ok, output "3 10 5 6"
use std.beats.{BeatsSegTree}
var t = BeatsSegTree([2, 9, 4, 6])
t.chmax(0..4, 3)
t.add(1..3, 1)
print(t)
