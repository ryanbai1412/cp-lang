// expect: ok, output "YES"
var h = Heap.of([3, 1, 2])
print(yes(h.pop() == Some(1) and h.len == 2))
