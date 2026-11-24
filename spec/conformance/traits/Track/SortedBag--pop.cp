// expect: ok, output "1 3"
use std.bag.{SortedBag}
fn evict<S: Track<Int>>(mut s: S, x: Int) = s.pop(x)
var b = SortedBag.of([3, 1, 3])
evict(mut b, 3)
print(b.to_list())
