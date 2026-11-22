// expect: R0401 at 2 (checked)
use std.geometry.{line_dist, Vec}
print(line_dist(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(0.0, 0.0)))
