// expect: ok, output "3 7 5"
use std.beats.{HistoricBeatsSegTree}
var t = HistoricBeatsSegTree([2, 8, 5])
t.chmax(0..3, 4)
t.add(0..2, -1)
print(t)
