// expect: ok, output "YES"
use std.bag.{SortedBag}
print(SortedBag.of([1, 4, 6]).first_true(x => x > 9) == None)
