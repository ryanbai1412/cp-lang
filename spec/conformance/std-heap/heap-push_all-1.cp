// expect: ok, output "YES"
var h = Heap.of([4])
h.push_all([3, 1])
print(yes(h.peek == Some(1) and h.len == 3))
