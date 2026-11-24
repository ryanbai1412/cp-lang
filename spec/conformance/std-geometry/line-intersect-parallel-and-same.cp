// expect: ok, output "YES YES"
use std.geometry.{Vec}
let p = line_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0))
let q = line_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(2.0, 0.0), Vec(3.0, 0.0))
print(yes(p == None), yes(q == None))
