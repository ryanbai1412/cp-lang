// expect: ok, output "5 YES"
// rule: std.heap.Heap.push_pop
var h = Heap<Int>()
print(h.push_pop(5), yes(h.is_empty))
