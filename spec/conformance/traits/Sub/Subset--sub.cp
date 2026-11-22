// expect: ok, output "3 NO"
use std.subset.{Subset}
domain B = 0..4
let a = Subset<B>.full - Subset<B>.single(B(3))
print(a.len, yes(B(3) in a))
