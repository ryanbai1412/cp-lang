// expect: ok, output "6 6 3 1 2 3"
// A std container walks through its `impl Seq`, so a fn bound by `Seq` takes it, and `each` is
// the ordinary std fn over any `Seq`.
use std.bag.{SortedSet}
fn total<S: Seq<Int>>(s: S) -> Int:
    var t = 0
    for x in s.iter(): t += x
    t
var s = SortedSet<Int>()
s.add(3)
s.add(1)
s.add(2)
let h = Heap.of([3, 1, 2])
print(total(h), total(s), h.each().collect().len, s.each().collect())
