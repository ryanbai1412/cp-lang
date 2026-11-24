// expect: R0301 (checked)
// rule: std.heap.Heap.pop
var h = Heap<Int>()
print(h.pop()!) // <- here
