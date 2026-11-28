// expect: ok, output "9 9\n6 6"
// rule: 08.1.4
// `sum` is a default method of `Seq` (spec 07 "A fold costs what the receiver's type says"): a
// type's own `sum` wins over it, and a type that writes none gets the default walk, through a
// `Seq` bound too.
use std.bag.{SortedBag}
fn total<S: Seq<Int>>(s: S) -> Int = s.sum()
let b = SortedBag.of([2, 2, 5])
print(b.sum(), total(b))
print([1, 2, 3].sum(), total([1, 2, 3]))
