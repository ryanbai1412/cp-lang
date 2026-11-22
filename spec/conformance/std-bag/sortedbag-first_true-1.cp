// expect: ok, output "YES"
use std.bag.{SortedBag}
print(yes(SortedBag.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)))
