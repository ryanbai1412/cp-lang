// expect: ok, output "3 7 7"
use std.bag.{BucketCount}
var c = BucketCount(0..10)
c.add(7, 2)
c.add(3)
print(c)
