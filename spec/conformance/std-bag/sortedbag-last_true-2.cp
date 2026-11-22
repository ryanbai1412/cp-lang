// expect: ok, output "YES"
use std.bag.{SortedBag}
print(yes(SortedBag.of([1, 4, 6]).last_true(x => x < 1) == None))
