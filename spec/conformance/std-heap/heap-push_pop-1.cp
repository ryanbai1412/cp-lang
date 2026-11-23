// expect: ok, output "YES"
var h = Heap.of([2, 5])
print(yes(h.push_pop(3) == 2 and h.peek == Some(3)))
