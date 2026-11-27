// expect: ok, output "3 2 3"
// rule: R05.5.3
// A walked element of a type parameter stored past its turn is a copy in each instance, not a borrow.
use std.seq.{Seq}
fn last_of<S: Seq<T>, T>(s: S) -> ?T:
    var prev: ?T = None
    for x in s: prev = Some(x)
    prev
let xs = [[1], [2, 3]]
print(last_of([1, 2, 3])!, last_of(xs)!.len, xs.len + 1)
