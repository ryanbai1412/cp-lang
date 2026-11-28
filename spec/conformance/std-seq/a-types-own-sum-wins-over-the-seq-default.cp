// expect: ok, output "9 9\n6 6"
// rule: 08.1.4
// A type's own `sum` wins over std's generic one, and a type that writes none gets the
// generic walk, through a `Seq` bound too.
use std.bag.{SortedBag}
fn total<S: Seq<Int>>(s: S) -> Int = s.sum()
let b = SortedBag.of([2, 2, 5])
print(b.sum(), total(b))
print([1, 2, 3].sum(), total([1, 2, 3]))
