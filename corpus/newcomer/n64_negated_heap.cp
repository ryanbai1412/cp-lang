let n: 1..=200000 = read()
let a: [0..n]Int = read()
var h = MinHeap<Int>()
for x in a:
    h.push(-x)  // expect: missing negated_heap
var top3 = 0
for _ in 0..min(3, n):  // expect: missing manual_heap_drain
    top3 += -(h.pop()!)
print(top3)
