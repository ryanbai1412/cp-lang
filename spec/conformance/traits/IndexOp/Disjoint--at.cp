// expect: ok, output "3 YES"
use std.ntt.{Disjoint}
print(Disjoint().at(1, 2)!, yes(Disjoint().at(3, 1) == None))
