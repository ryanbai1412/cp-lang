// expect: ok, output "YES"
var h = Heap.of([5])
h.push(2)
print(yes(h.peek == Some(2)))
