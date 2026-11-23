// expect: ok, output "YES"
var h = Heap.of([2, 1])
let t = h.take()
print(yes(h.is_empty and t.peek == Some(1)))
