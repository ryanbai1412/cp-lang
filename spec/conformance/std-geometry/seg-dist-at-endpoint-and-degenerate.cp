// expect: ok, output "YES YES"
use std.geometry.{seg_dist, Vec}
print(yes(seg_dist(Vec(2.0, 0.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 0.0), yes(seg_dist(Vec(3.0, 4.0), Vec(0.0, 0.0), Vec(0.0, 0.0)) == 5.0))
