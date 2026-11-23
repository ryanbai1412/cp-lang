// expect: ok, output "3 11 15\n2 3 14 16"
// A pending `add_all` offset reaches every element it was added to, through later pushes and melds.
use std.heap.{MeldHeap}
var h = MeldHeap.of([1, 5])
h.add_all(10)
h.push(3)
print(h.drain())
var a = MeldHeap.of([4, 6])
a.add_all(10)
var b = MeldHeap.of([2, 3])
b.meld(a)
print(b.drain())
