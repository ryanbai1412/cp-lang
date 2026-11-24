// expect: ok, output "5"
use std.subset.{Subset}
domain B = 0..4
print(Int(Subset<B>.single(B(0)) | Subset<B>.single(B(2))))
