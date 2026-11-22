// expect: ok, output "0 0"
use std.slope.{Concave}
var f = Concave.flat(0..=4)
f.suffix_max()
print(f.at(0), f.at(4))
