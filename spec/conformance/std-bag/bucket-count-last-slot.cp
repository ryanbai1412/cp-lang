// expect: ok, output "1 1"
// rule: std.bag.BucketCount.add
use std.bag.{BucketCount}
var b = BucketCount(0..10)
b.add(9)
print(b.count(9), b.len)
