// expect: ok, output "1 4 9\n9 4\n1 3 4"
// rule: std.heap push_all
// `push_all` takes any sequence, such as a filter or a map, not only a `List`.
use std.heap.{Heap, MeldHeap}
var a = MeldHeap.of([4])
a.push_all([1, 2, 3].map(x => x * x).filter(x => x != 4))
print(a.drain())
var b = MaxHeap<Int>()
b.push_all([2, 9, 4].filter(x => x > 3))
print([b.pop()!, b.pop()!])
var c = Heap.of([4])
c.push_all((1..4).filter(x => x != 2))
print([c.pop()!, c.pop()!, c.pop()!])
