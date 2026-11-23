// expect: ok, output "-1 1 -1"
var h = MinHeap<Int>()
let first = h.pop() ?? -1
h.push(1)
print(first, h.pop() ?? -1, h.pop() ?? -1)
