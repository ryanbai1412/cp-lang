// expect: ok, output "2"
use std.bag.{SortedBag}
fn add<S: Track<Int>>(mut s: S, x: Int) = s.push(x)
var b: SortedBag<Int> = SortedBag()
add(mut b, 3)
add(mut b, 3)
print(b.len)
