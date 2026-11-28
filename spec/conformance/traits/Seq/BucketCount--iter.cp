// expect: ok, output "3\n7\n7"
// engines: next
use std.bag.{BucketCount}
var c = BucketCount(0..10)
c.add(7, 2)
c.add(3)
for x in c: print(x)
