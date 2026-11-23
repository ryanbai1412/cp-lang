// expect: ok, output "5"
// rule: std.heap.Heap
// A descending key written as a lambda works like the bare `desc` (rebuild round 18 B2).
var h: Heap<Int> = Heap(by: x => desc(x))
h.push(1)
h.push(5)
print(h.pop()!)
