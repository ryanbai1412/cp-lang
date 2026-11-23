// expect: ok, output "YES"
var h = Heap.of([2, 5])
print(yes(h.replace_top(9) == Some(2) and h.peek == Some(5)))
