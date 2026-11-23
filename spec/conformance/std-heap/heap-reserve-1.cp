// expect: ok, output "YES"
var h = Heap.of([2])
h.reserve(100)
print(yes(h.peek == Some(2)))
