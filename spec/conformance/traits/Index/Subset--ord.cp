// expect: ok, output "6 8"
use std.subset.{Subset}
domain B = 0..3
print(Subset<B>.single(B(1)).with(B(2)).ord, Subset<B>.count)
