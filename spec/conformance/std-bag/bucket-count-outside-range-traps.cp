// expect: R0401 (checked)
// rule: std.bag.BucketCount.add
use std.bag.{BucketCount}
var b = BucketCount(0..10)
b.add(10) // <- here
print(b.len)
