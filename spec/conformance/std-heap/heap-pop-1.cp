// expect: ok, output "YES"
var h = Heap.of([2, 1])
print(yes(h.pop() == Some(1) and h.pop() == Some(2) and h.pop() == None))
