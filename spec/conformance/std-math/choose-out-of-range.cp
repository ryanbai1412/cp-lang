// expect: ok, output "1 0 0 1"
use std.math.{choose}
print(choose(0, 0), choose(3, 4), choose(3, -1), choose(7, 7))
